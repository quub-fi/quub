# Quub status (Sprint 6)

## Dual-control / owners

| Environment | OwnerA | OwnerB | Checker proof |
|---|---|---|---|
| `--dev` (8091, `alloc_genesis.rs`) | anvil0 `0xf39F…` | anvil1 `0x7099…` | Live: `scripts/sprint5-dev.sh` propose+confirm |
| Mode A L2 (op-deployer artifacts) | **unchanged** (may still be shared) | same | **Foundry** `test_unfreeze_proposeA_confirmB` owns the checker proof |

Do not treat Mode A shared owners as dual-control. Do not ship “two calls from the same key” as maker-checker.

## Paymaster (ADR-019)

- Fee only on `transferWithMemo` (plain `transfer` fee-free).
- F213 `feeRecipient` = `0x…FEE0` on `--dev` (slot 1; formerly `treasury`).
- Flat quote constants on F210 (`FEE_GAS_LIMIT` / `FEE_GAS_PRICE`); no `tx.gasprice`.
- F203 stays thin; F213 owns the ERC-20 move via `paymasterDebit`.

## CCTP Base rail (ADR-020)

- Quub **8091** records identity (`transferWithMemo`). Circle CCTP V2 burns USDC on **Ethereum Sepolia** (domain 0) and mints on **Base Sepolia** (domain 6).
- Quub is not a CCTP domain. Burn id must never equal Quub tx hash.
- Pins: `specs/rail-cctp.md`. Mock is a valid pass without Circle env.
- Grade: `scripts/rail-base.sh` → `mode=live|mock` + Quub tx + memoHash + burn id + mint id.
- Solana adapter remains a stub. `operator-api` `/v1/rails/cctp` out of scope.

## Workspace

`apps/operator-api` (Sprint 4 WIP) may fail `cargo test --workspace`. Sprint 6 grades with:

`cargo test --workspace --exclude quub-operator-api`

- Instant `unfreeze(address)` removed from PolicyAdmin.
- `scripts/devnet.sh` does not unfreeze; use `scripts/sprint5-dev.sh`.
