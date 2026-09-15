# ADR-019 — Paymaster live + dual-control stub

**Status:** Accepted (Sprint 5)  
**Date:** 2026-09-15  
**Depends on:** ADR-009 (frozen addresses), ADR-016 (pins unchanged)

## Decision

1. **Fee on memo only.** `PaymentToken.transferWithMemo` takes a flat fee via F213 after policy allow and **before** principal `_move`. Plain `transfer` / `transferFrom` stay fee-free.
2. **F213 owns the move.** `takeFee` is callable **only by F210**. Debit is `paymasterDebit` → `_move` (no open `transferFrom`, no nested policy/memo/takeFee). Storage field `feeRecipient` (slot 1, formerly `treasury`); `--dev` sink = `0x…FEE0` (not anvil0, so fee is observable).
3. **Flat quote.** F210 uses constants `FEE_GAS_LIMIT` / `FEE_GAS_PRICE` (not `tx.gasprice`) so `quote == taken ±1 wei` on `--dev`.
4. **Maker-checker on F211.** `freeze` is immediate from either owner. `unfreeze` / `setFeeToken` / `setThreshold` / `pause` are propose + confirm by the **other** owner. Same key twice does not count. No public instant `unfreeze`.
5. **`--dev` owners.** OwnerA = anvil0, OwnerB = anvil1. Mode A L2 genesis is **not** regenerated this sprint (see `STATUS.md`).

## Rejected

- 24h OpenZeppelin timelock / proxies / EIP-4337
- Fee on freeze or on plain transfer
- Retuning `PAYMENT_LANE_BPS` because memo fee gas changed flood share
- F203 stateful ERC-20 debit (stays thin)
- Instant `unfreeze` “compatibility” shim

## Consequences

- Genesis runtime hex for F210/F211/F213 regenerated; slot 1 on F213 remains the fee sink (`0x…FEE0` on `--dev`).
- Live grade: `scripts/sprint5-dev.sh`. Flood: `scripts/payment-lane-flood.sh` must still run.
