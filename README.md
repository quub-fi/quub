# Quub

Quub is a payments fabric. There is no native token.

Quub is the payments fabric: ledger (Rust node + EVM + Solidity facades) plus Quub Policy, Quub Memo / Quub ISO, Quub Evidence, and Quub Gateway. **fazeZERO** is the company and is not in this repository.

## Pins (ADR-016)

| Component | Pin |
|-----------|-----|
| Reth | `op-rs/reth` rev **`aef8d3ef92117f91455e16969f0adf5bf7c6e9e1`** |
| OP crates | `ethereum-optimism/optimism` tag **`op-reth/v2.4.4`** |
| op-node | **v1.19.7** (minimum v1.19.1) |
| Rust toolchain | **1.95.0** |
| L2 chain id | **8091** (`--dev` and Mode A) |

One Reth remote only. See [`docs/adr/ADR-016-execution-pin.md`](docs/adr/ADR-016-execution-pin.md).

### Install op-node (Mode A)

```bash
# Example: Go install from the optimism monorepo at the op-node tag
git clone https://github.com/ethereum-optimism/optimism.git
cd optimism && git checkout op-node/v1.19.7
cd op-node && just op-node  # or: go build -o op-node ./cmd/node
export PATH="$(pwd):$PATH"
op-node --version   # expect v1.19.7
```

Also require **op-deployer** (or the official local-dev path for that op-node tag) before `scripts/mode-a.sh`. Do not hand-write `rollup.json` or invent a portal address.

## PATH for `quub-node`

```bash
cargo build -p quub-node
export PATH="$(pwd)/target/debug:$PATH"
quub-node   # --dev HTTP on 8545
```

Or use `cargo run -p quub-node` / `bash scripts/devnet.sh` without installing.

## `--dev` (Sprint 1.5+)

- `quub-node` launches an `EthereumNode` with `QuubExecutorBuilder` (no Simplex).
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

## Mode A (`--engine`)

- L2 HTTP **9545**, authrpc **9551**, JWT file, chain id **8091**. Not Sepolia.
- Uses Optimism node types + the same Quub precompiles. No `--dev` miner on this process.
- Genesis / `rollup.json` / L1 portal from **op-deployer** (or official local-dev for op-node/v1.19.7), then overlay F210–F213. Never invent a portal address.

```bash
bash scripts/mode-a.sh
```

## Locked stack

Rust + Solidity + Foundry. Mode A (OP Stack) is the default crate feature (`mode-a`). Mode B (Simplex) is feature-flagged and never enabled in the same binary. Not Substrate. Not a custom VM.
