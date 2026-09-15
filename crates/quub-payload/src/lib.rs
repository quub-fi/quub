//! Payment-lane block fill (Sprint 3 / ADR-017).
//!
//! Pure algorithm: no Reth, no consensus. Adapters reorder pool candidates and
//! hand the ordered list to stock payload builders.

use quub_primitives::{payment_gas_budget, PAYMENT_LANE_BPS};

/// One pool/candidate tx for lane selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneCandidate<Id> {
    pub id: Id,
    pub is_payment: bool,
    /// Declared gas limit (packing slack vs executed gas is expected).
    pub gas_limit: u64,
    /// Tip for general-lane ordering (higher first). Ignored for payments (FIFO).
    pub tip: u128,
}

/// Fill order: payments up to `bps` → general up to remainder → leftover payments then general.
///
/// Empty payment lane may consume the full block with general (no hollow 70%).
/// Lane beats tip: high-tip generals cannot steal reserved payment gas while payments wait.
pub fn fill_lanes<Id: Clone>(
    block_gas: u64,
    payments: &[LaneCandidate<Id>],
    generals: &[LaneCandidate<Id>],
    bps: u16,
) -> Vec<Id> {
    let payment_cap = (block_gas as u128 * bps as u128 / 10_000) as u64;
    let general_cap = block_gas.saturating_sub(payment_cap);

    let mut selected: Vec<Id> = Vec::new();
    let mut payment_used = 0u64;
    let mut general_used = 0u64;
    let mut remaining = block_gas;
    let mut pay_taken = vec![false; payments.len()];
    let mut gen_taken = vec![false; generals.len()];

    // Phase 1: payments FIFO into payment budget.
    for (i, p) in payments.iter().enumerate() {
        if p.gas_limit == 0 || p.gas_limit > remaining {
            continue;
        }
        if payment_used + p.gas_limit > payment_cap {
            continue;
        }
        selected.push(p.id.clone());
        payment_used += p.gas_limit;
        remaining -= p.gas_limit;
        pay_taken[i] = true;
    }

    // Phase 2: generals by tip into general budget (caller tip-sorts descending).
    for (i, g) in generals.iter().enumerate() {
        if g.gas_limit == 0 || g.gas_limit > remaining {
            continue;
        }
        if general_used + g.gas_limit > general_cap {
            continue;
        }
        selected.push(g.id.clone());
        general_used += g.gas_limit;
        remaining -= g.gas_limit;
        gen_taken[i] = true;
    }

    // Phase 3 spill: remaining payments, then remaining generals.
    for (i, p) in payments.iter().enumerate() {
        if pay_taken[i] {
            continue;
        }
        if p.gas_limit == 0 || p.gas_limit > remaining {
            continue;
        }
        selected.push(p.id.clone());
        remaining -= p.gas_limit;
        pay_taken[i] = true;
    }
    for (i, g) in generals.iter().enumerate() {
        if gen_taken[i] {
            continue;
        }
        if g.gas_limit == 0 || g.gas_limit > remaining {
            continue;
        }
        selected.push(g.id.clone());
        remaining -= g.gas_limit;
        gen_taken[i] = true;
    }

    let _ = (PAYMENT_LANE_BPS, payment_gas_budget(block_gas));
    selected
}

/// Sort generals by tip descending (stable for equal tips).
pub fn sort_generals_by_tip<Id: Clone>(generals: &mut [LaneCandidate<Id>]) {
    generals.sort_by(|a, b| b.tip.cmp(&a.tip));
}

/// Split candidates into payment (FIFO) and general (tip-desc), then [`fill_lanes`].
pub fn order_lane_candidates<Id: Clone>(
    block_gas: u64,
    candidates: Vec<LaneCandidate<Id>>,
) -> Vec<Id> {
    let mut payments = Vec::new();
    let mut generals = Vec::new();
    for c in candidates {
        if c.is_payment {
            payments.push(c);
        } else {
            generals.push(c);
        }
    }
    sort_generals_by_tip(&mut generals);
    fill_lanes(block_gas, &payments, &generals, PAYMENT_LANE_BPS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quub_primitives::payment_gas_budget;

    fn pay(id: &'static str, gas: u64) -> LaneCandidate<&'static str> {
        LaneCandidate {
            id,
            is_payment: true,
            gas_limit: gas,
            tip: 0,
        }
    }

    fn gen(id: &'static str, gas: u64, tip: u128) -> LaneCandidate<&'static str> {
        LaneCandidate {
            id,
            is_payment: false,
            gas_limit: gas,
            tip,
        }
    }

    #[test]
    fn mixed_fills_seventy_thirty() {
        let block = 10_000u64;
        let payments = [
            pay("p0", 1000),
            pay("p1", 1000),
            pay("p2", 1000),
            pay("p3", 1000),
            pay("p4", 1000),
            pay("p5", 1000),
            pay("p6", 1000),
        ];
        let mut generals = [
            gen("g0", 1000, 10),
            gen("g1", 1000, 20),
            gen("g2", 1000, 30),
        ];
        sort_generals_by_tip(&mut generals);
        let order = fill_lanes(block, &payments, &generals, PAYMENT_LANE_BPS);
        assert_eq!(order.len(), 10);
        assert_eq!(&order[..7], &["p0", "p1", "p2", "p3", "p4", "p5", "p6"]);
        assert_eq!(&order[7..], &["g2", "g1", "g0"]);
    }

    #[test]
    fn no_payments_general_may_fill_full_block() {
        let block = 10_000u64;
        let payments: [LaneCandidate<&str>; 0] = [];
        let mut generals = [
            gen("g0", 1000, 10),
            gen("g1", 1000, 9),
            gen("g2", 1000, 8),
            gen("g3", 1000, 7),
            gen("g4", 1000, 6),
            gen("g5", 1000, 5),
            gen("g6", 1000, 4),
            gen("g7", 1000, 3),
            gen("g8", 1000, 2),
            gen("g9", 1000, 1),
        ];
        sort_generals_by_tip(&mut generals);
        let order = fill_lanes(block, &payments, &generals, PAYMENT_LANE_BPS);
        assert_eq!(order.len(), 10, "empty payment lane must not leave hollow 70%");
    }

    #[test]
    fn no_general_payments_only() {
        let block = 10_000u64;
        let payments = [pay("p0", 1000), pay("p1", 1000), pay("p2", 1000)];
        let generals: [LaneCandidate<&str>; 0] = [];
        let order = fill_lanes(block, &payments, &generals, PAYMENT_LANE_BPS);
        assert_eq!(order, ["p0", "p1", "p2"]);
    }

    #[test]
    fn huge_general_cannot_steal_seventy() {
        // Product test: high-tip generals must not displace payments from first 70%.
        let block = 10_000u64;
        let payments = [
            pay("p0", 2000),
            pay("p1", 2000),
            pay("p2", 2000),
            pay("p3", 1000),
        ];
        let mut generals = [
            gen("whale", 5000, 1_000_000),
            gen("g1", 1000, 999_999),
            gen("g2", 1000, 999_998),
            gen("g3", 1000, 999_997),
        ];
        sort_generals_by_tip(&mut generals);
        let order = fill_lanes(block, &payments, &generals, PAYMENT_LANE_BPS);

        assert_eq!(&order[..4], &["p0", "p1", "p2", "p3"]);

        let mut payment_gas = 0u64;
        for id in &order {
            if let Some(p) = payments.iter().find(|p| p.id == *id) {
                payment_gas += p.gas_limit;
            }
        }
        assert_eq!(payment_gas, 7000);

        let reserved = payment_gas_budget(block);
        for (i, id) in order.iter().enumerate() {
            if id.starts_with('p') {
                continue;
            }
            let gas_before: u64 = order[..i]
                .iter()
                .filter_map(|x| payments.iter().find(|p| p.id == *x).map(|p| p.gas_limit))
                .sum();
            assert!(
                gas_before >= reserved,
                "general {id} at index {i} stole payment reserved gas (payments so far {gas_before})"
            );
        }
    }
}
