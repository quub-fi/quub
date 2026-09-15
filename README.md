# Quub

Quub is a payments fabric. There is no native token.

Quub is the ledger (Rust node + EVM + Solidity facades). **xZERO** is the off-chain orchestration plane (policy, ISO 20022 memos, evidence, multi-rail routing). **fazeZERO** is the company and is not in this repository.

## Sprint 0

This workspace currently delivers Sprint 0 only:

- `crates/quub-primitives` — frozen addresses, reason codes, memo types
- `services/xzero-*` — off-chain policy / ISO / evidence / rail stubs
- `contracts/` — Foundry system contracts and tests

No Reth node, no consensus, no custom transaction type.

## Build / test

```bash
cargo test --workspace
cd contracts && forge test
```

## Locked stack

Rust + Solidity + Foundry. Mode A (OP Stack) is the default consensus path in Sprint 1+. Mode B (Simplex) is feature-flagged and never enabled in the same binary. Not Substrate. Not a custom VM.
