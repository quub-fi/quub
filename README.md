# Quub

Quub is a payments fabric. There is no native token.

Quub is the ledger (Rust node + EVM + Solidity facades). **xZERO** is the off-chain orchestration plane (policy, ISO 20022 memos, evidence, multi-rail routing). **fazeZERO** is the company and is not in this repository.

## Sprint 1

- Reth pin: `paradigmxyz/reth` git tag **`v2.5.2`** (`reth-ethereum` features `node`, `evm`).
- Rust toolchain: **1.95.0** (Reth 2.5.2 MSRV).
- `quub-node` launches an `EthereumNode` with `QuubExecutorBuilder` (no `op-node`, no Simplex).
- HTTP JSON-RPC: `http://127.0.0.1:8545`, chain id **8091**.
- Precompiles F201–F203 are registered for `spec >= Prague`.

## Build / test

```bash
cargo test --workspace
cd contracts && forge test
```

## Local node + F202 `cast call`

```bash
# terminal 1
cargo run -p quub-node

# terminal 2
cast chain-id --rpc-url http://127.0.0.1:8545          # expect 8091
cast call 0x000000000000000000000000000000000000F202 \
  "validateAndCommit(bytes32,bytes16,bytes32,bytes3,uint8)(bytes32)" \
  0x1111111111111111111111111111111111111111111111111111111111111111 \
  0x22222222222222222222222222222222 \
  0x3333333333333333333333333333333333333333333333333333333333333333 \
  0x555344 0 \
  --rpc-url http://127.0.0.1:8545
# must return a 32-byte hash, not revert-empty
```

Or: `bash scripts/devnet.sh`.

## Locked stack

Rust + Solidity + Foundry. Mode A (OP Stack) is the default **crate feature** (`mode-a` compiles a stub; launch does not call it). Mode B (Simplex) is feature-flagged and never enabled in the same binary. Not Substrate. Not a custom VM.
