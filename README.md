# Quub

Quub is a payments fabric. There is no native token.

Quub is the payments fabric: ledger (Rust node + EVM + Solidity facades) plus Quub Policy, Quub Memo / Quub ISO, Quub Evidence, and Quub Gateway. **fazeZERO** is the company and is not in this repository.

## Pins

| Component | Pin |
|-----------|-----|
| Reth | `paradigmxyz/reth` git tag **`v2.5.2`** |
| Rust toolchain | **1.95.0** |
| L2 chain id | **8091** (same for `--dev` and Mode A when Mode A is unblocked) |
| op-node / op-reth | **blocked** — see [Mode A version conflict](#mode-a-version-conflict-sprint-2) |

## PATH for `quub-node`

```bash
cargo build -p quub-node
export PATH="$(pwd)/target/debug:$PATH"
quub-node   # --dev HTTP on 8545
```

Or use `cargo run -p quub-node` / `bash scripts/devnet.sh` without installing.

## Sprint 1.5 / `--dev`

- `quub-node` launches an `EthereumNode` with `QuubExecutorBuilder` (no `op-node`, no Simplex).
- HTTP JSON-RPC: `http://127.0.0.1:8545`, chain id **8091**.
- Precompiles F201–F203 registered for `spec >= Prague` (`new_stateful` for F201/F202).
- System contracts etched at genesis: **F210–F213** (see `crates/quub-node/alloc/`).
- Datadir is ephemeral (`NodeConfig::test()`); receipts do not survive a restart. The node prints the path on stderr.

### --dev key (public anvil account 0 — not production)

- Address: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`
- Private key: `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`
- PolicyAdmin OwnerA = OwnerB = this key is a **`--dev` bypass**. Production dual-control + 24h timelock is later.

### Frozen addresses

| Addr | Role |
|------|------|
| `0x…F201` | Quub Policy precompile |
| `0x…F202` | Quub ISO memo precompile |
| `0x…F203` | Quub Paymaster precompile |
| `0x…F210` | PaymentToken |
| `0x…F211` | PolicyAdmin |
| `0x…F212` | EvidenceAnchor |
| `0x…F213` | PaymasterEntry |

On `--dev`, `cast codesize 0x…F213` is non-zero (genesis etch of `PaymasterEntry.runtime.hex`). Genesis does **not** skip F213.

### Memo hash origins

`memoHash = keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, tx.origin))`

- Fixture `usd_pacs008_stable_hash` uses origin `0x…00AA`.
- A mined `transferWithMemo` from the anvil key uses origin `0xf39F…` — **not** equal to the fixture.
- `cast call` F202 `--from 0xf39F…` must match the receipt `MemoAnchored` hash.

## Build / test

```bash
cargo test --workspace
cd contracts && forge test
```

## Local payment (`--dev`)

```bash
bash scripts/devnet.sh
```

Or manually:

```bash
cargo build -p quub-node && export PATH="$(pwd)/target/debug:$PATH"
quub-node   # terminal 1 — HTTP 8545

cast chain-id --rpc-url http://127.0.0.1:8545   # expect 8091
cast codesize 0x000000000000000000000000000000000000F213 --rpc-url http://127.0.0.1:8545
cast send 0x000000000000000000000000000000000000F210 \
  "transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)" \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  1000000 \
  0x1111111111111111111111111111111111111111111111111111111111111111 \
  0x22222222222222222222222222222222 \
  0x3333333333333333333333333333333333333333333333333333333333333333 \
  0x555344 \
  0 \
  0x0000000000000000000000000000000000000000000000000000000000000000 \
  0x4444444444444444444444444444444444444444444444444444444444444444 \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --legacy --gas-price 1000000000 --gas-limit 500000
```

Send to **F210**, not F201. Raw `cast call` F201 from a non-F210 caller empty-reverts. Use a full `bytes32` zero (`0x0000…0000`), not `0x0`.

## Mode A version conflict (Sprint 2)

**Stopped without bumping Reth.** Facts:

1. Quub pins **`paradigmxyz/reth` `v2.5.2`** (`Cargo.toml` workspace dep). That tree has **no** `crates/optimism` / `reth-optimism-*` packages (op-reth was removed from the Reth monorepo).
2. Current op-reth lives in **`ethereum-optimism/optimism`** (`rust/op-reth/`) and pins **`op-rs/reth`**, not `paradigmxyz/reth` `v2.5.2`. Latest published tag observed: **`op-reth/v2.4.4`**.
3. Wiring `quub-consensus-op` to OP Engine API + deposit types therefore cannot compile against the locked Quub Reth pin without either bumping / switching the Reth source (forbidden this sprint) or vendoring a second Reth (forbidden: do not fork Reth).

See `crates/quub-consensus-op/CONFLICT.md`. `scripts/mode-a.sh` exits with this conflict. `quub-node --engine` prints the same and exits non-zero. **`--dev` on 8545 remains the working path.**

When unblocked: L2 HTTP **9545**, authrpc **9551**, JWT file, same F210–F213 alloc, chain id **8091**. Not Sepolia.

```bash
bash scripts/mode-a.sh   # currently exits: Mode A blocked
```

## Locked stack

Rust + Solidity + Foundry. Mode A (OP Stack) is the default **crate feature** (`mode-a`); launch does not start `op-node` until the Reth pin conflict is resolved by a human decision. Mode B (Simplex) is feature-flagged and never enabled in the same binary. Not Substrate. Not a custom VM.
