# ADR-020 — CCTP Base rail (Quub records; Circle moves)

**Status:** Accepted (Sprint 6)  
**Date:** 2026-09-15  
**Depends on:** ADR-012 (adapters + official transports), ADR-013 (USDC = CCTP V2), ADR-016 (pins unchanged)

## Decision

1. **Quub records; Circle moves USDC.** On chain **8091**, Quub mines `transferWithMemo` (policy + memo + evidence identity). Circle **CCTP V2** burns native USDC on **Ethereum Sepolia** (domain **0**) and mints on **Base Sepolia** (domain **6**). Quub is not a bridge.
2. **8091 is not a CCTP domain.** `cast send` on `:8545` is Quub identity only. The burn tx must **never** equal the Quub tx hash. `RailRecord` holds both worlds (`quub_tx`, `memo_hash`, `burn_tx`, `mint_tx`, `status`).
3. **V2 messengers only.** Pin TokenMessengerV2 / MessageTransmitterV2 / USDC from Circle’s current testnet table in `specs/rail-cctp.md`. Mock uses the **same** address/domain struct.
4. **Policy reject → no burn.** CCTP fail after Quub mined → `status=failed`, **keep** the memo; retry CCTP with the same `endToEndId` (no second Quub memo, no double-burn).
5. **Mock is a valid pass** when `SEPOLIA_RPC` / `BASE_SEPOLIA_RPC` / `CIRCLE_API` / `CCTP_BURNER_KEY` are unset. Live requires those env vars and ≤1 USDC on Sepolia.

## Rejected

- Wrapped USDC / Quub-canonical dollar / new F2xx
- CCIP for this USDC hop; Solana rail this sprint
- Treating Quub 8091 as a CCTP source domain
- Packing Quub tx hash into TokenMessenger
- Blocking on `operator-api` `/v1/rails/cctp`
- Editing payment lane / Reth pin / F211 for this rail

## Consequences

- `services/quub-gateway`: `CctpAdapter` + `RailRecord` / `RailFlow`; Solana stub untouched.
- Grade: `scripts/rail-base.sh` prints `mode=live|mock`, Quub tx, memoHash, burn id, mint id; fails if burn == Quub tx.
