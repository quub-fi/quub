# Quub status (Sprint 5)

## Dual-control / owners

| Environment | OwnerA | OwnerB | Checker proof |
|---|---|---|---|
| `--dev` (8091, `alloc_genesis.rs`) | anvil0 `0xf39F…` | anvil1 `0x7099…` | Live: `scripts/sprint5-dev.sh` propose+confirm |
| Mode A L2 (op-deployer artifacts) | **unchanged this sprint** (may still be shared) | same | **Foundry** `test_unfreeze_proposeA_confirmB` owns the checker proof |

Do not treat Mode A shared owners as dual-control. Do not ship “two calls from the same key” as maker-checker.

## Paymaster (ADR-019)

- Fee only on `transferWithMemo` (plain `transfer` fee-free).
- F213 `feeRecipient` = `0x…FEE0` on `--dev` (slot 1; formerly `treasury`).
- Flat quote constants on F210 (`FEE_GAS_LIMIT` / `FEE_GAS_PRICE`); no `tx.gasprice`.
- F203 stays thin; F213 owns the ERC-20 move via `paymasterDebit`.

## Workspace

`apps/operator-api` (Sprint 4 WIP) may fail `cargo test --workspace`. Sprint 5 grades with:

`cargo test --workspace --exclude quub-operator-api`

- Instant `unfreeze(address)` removed from PolicyAdmin.
- `scripts/devnet.sh` does not unfreeze; use `scripts/sprint5-dev.sh`.
