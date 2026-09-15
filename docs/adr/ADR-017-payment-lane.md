# ADR-017 — Payment lane (70/30 fill)

**Status:** Accepted (Sprint 3)  
**Date:** 2026-09-15  
**Depends on:** ADR-011 (no new EIP-2718 type), ADR-016 (Reth pin unchanged)

## Decision

Quub classifies and packs payment traffic **without** a new transaction type.

| Piece | Rule |
|-------|------|
| Classifier | `to == F210` **and** calldata is exactly `transferWithMemo` (selector `0x843e111e` + length `4+9×32`). Plain `transfer` on F210 is **general**. |
| Fill order | Payment FIFO up to `PAYMENT_LANE_BPS` (7000 = 70%) of block gas → general tip-ordered to 30% → spill leftover payments then general. Empty payment lane may fill **100% general**. |
| Tip | **Lane beats tip.** A high-tip general must not land inside the reserved 70% while payments wait. |
| Deposits | Engine-injected (Mode A); not pool-lane traffic. |
| Builders | Same `quub_payload::fill_lanes` / `order_lane_candidates` on `--dev` (`QuubEthPayloadBuilder`) and `--engine` (`QuubLaneTxs: OpPayloadTransactions`). Stock payload builders only — reorder candidates, do not invent envelopes. |

## Rules

1. **No new EIP-2718 type.** If the payload trait fight starts, reorder pool candidates and hand an ordered list to the stock builder.
2. Do not bump Reth past ADR-016 to make lane “first-class.”
3. Live flood acceptance is **≥ ~65%** payment gas under payment-heavy demand (packing slack) — not a perfect 70.000%.
4. Do not put 70/30 into PolicyAdmin. Do not enable Mode B for this.

## Why

Payments need reserved blockspace so fee-market tip wars cannot starve `transferWithMemo`. Prefix-only classifiers are forbidden (AGENTS §5); memo-only on F210 matches year-1 product traffic.

## Consequences

- `quub-pool` / `quub-payload` / `quub-node` launch payload wiring and `quub-consensus-op::QuubLaneTxs` own the lane while this ADR holds.
- Grade via `scripts/payment-lane-flood.sh` on 8545 (hash, counts, gas, share). Mode A 9545 flood is optional; code-path proof is required.
