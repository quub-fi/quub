//! Off-chain policy engine matching F201 `check` ABI.
//!
//! Year-1 behaviour mirrors PolicyAdmin: fail closed on freeze / pause /
//! missing Travel Rule when required. Only the payment-token path should call
//! this; malformed inputs return reason 8.

use alloy_primitives::{Address, B256, U256};
use quub_primitives::{PolicyCheck, PolicyReason};

/// Mutable policy state used by the off-chain engine (and later by F211).
#[derive(Clone, Debug, Default)]
pub struct PolicyState {
    pub paused: bool,
    pub frozen: std::collections::HashSet<Address>,
    pub allowlisted: std::collections::HashSet<Address>,
    pub amount_limit: Option<U256>,
    /// When true, a zero `trHash` is rejected with `MissingTravelRule`.
    pub require_travel_rule: bool,
    pub dual_control_required: bool,
}

impl PolicyState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn freeze(&mut self, addr: Address) {
        self.frozen.insert(addr);
    }

    pub fn unfreeze(&mut self, addr: Address) {
        self.frozen.remove(&addr);
    }

    pub fn allowlist(&mut self, addr: Address) {
        self.allowlisted.insert(addr);
    }
}

/// Result of `check`: `(allowed, reason)`.
pub type CheckResult = (bool, PolicyReason);

/// F201 `check(token, from, to, amount, trHash)`.
pub fn check(state: &PolicyState, req: &PolicyCheck) -> CheckResult {
    // Malformed: zero addresses or zero amount.
    if req.from.is_zero() || req.to.is_zero() || req.token.is_zero() || req.amount.is_zero() {
        return (false, PolicyReason::Malformed);
    }

    if state.paused {
        return (false, PolicyReason::Paused);
    }

    if state.frozen.contains(&req.from) {
        return (false, PolicyReason::FrozenFrom);
    }

    if state.frozen.contains(&req.to) {
        return (false, PolicyReason::FrozenTo);
    }

    // Empty allowlist means "all allowed" for year-1 off-chain stub;
    // once populated, both from and to must be listed.
    if !state.allowlisted.is_empty()
        && (!state.allowlisted.contains(&req.from) || !state.allowlisted.contains(&req.to))
    {
        return (false, PolicyReason::NotAllowlisted);
    }

    if let Some(limit) = state.amount_limit {
        if req.amount > limit {
            return (false, PolicyReason::AmountOverLimit);
        }
    }

    if state.require_travel_rule && req.tr_hash == B256::ZERO {
        return (false, PolicyReason::MissingTravelRule);
    }

    if state.dual_control_required {
        return (false, PolicyReason::DualControlRequired);
    }

    (true, PolicyReason::Allow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, uint};

    fn sample_req() -> PolicyCheck {
        PolicyCheck {
            token: address!("0x000000000000000000000000000000000000F210"),
            from: address!("0x0000000000000000000000000000000000000001"),
            to: address!("0x0000000000000000000000000000000000000002"),
            amount: uint!(100_U256),
            tr_hash: B256::ZERO,
        }
    }

    #[test]
    fn reason_0_allow() {
        let state = PolicyState::new();
        let (ok, reason) = check(&state, &sample_req());
        assert!(ok);
        assert_eq!(reason, PolicyReason::Allow);
        assert_eq!(reason.as_u16(), 0);
    }

    #[test]
    fn reason_1_frozen_from() {
        let mut state = PolicyState::new();
        let req = sample_req();
        state.freeze(req.from);
        let (ok, reason) = check(&state, &req);
        assert!(!ok);
        assert_eq!(reason, PolicyReason::FrozenFrom);
        assert_eq!(reason.as_u16(), 1);
    }

    #[test]
    fn reason_2_frozen_to() {
        let mut state = PolicyState::new();
        let req = sample_req();
        state.freeze(req.to);
        let (ok, reason) = check(&state, &req);
        assert!(!ok);
        assert_eq!(reason, PolicyReason::FrozenTo);
        assert_eq!(reason.as_u16(), 2);
    }

    #[test]
    fn reason_5_missing_travel_rule() {
        let mut state = PolicyState::new();
        state.require_travel_rule = true;
        let req = sample_req();
        let (ok, reason) = check(&state, &req);
        assert!(!ok);
        assert_eq!(reason, PolicyReason::MissingTravelRule);
        assert_eq!(reason.as_u16(), 5);
    }

    #[test]
    fn reason_8_malformed() {
        let state = PolicyState::new();
        let mut req = sample_req();
        req.from = Address::ZERO;
        let (ok, reason) = check(&state, &req);
        assert!(!ok);
        assert_eq!(reason, PolicyReason::Malformed);
        assert_eq!(reason.as_u16(), 8);
    }
}
