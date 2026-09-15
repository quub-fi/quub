# ADR-016 — Execution pin (Mode A)

**Status:** Accepted (Sprint 2)  
**Date:** 2026-09-15  
**Supersedes for pins:** ADR-4’s paradigmxyz/reth `v2.5.2` line (library rule unchanged)

## Decision

Quub’s execution library and OP Stack crates use **one** Reth remote and the optimism op-reth tag that matches it. Pair with op-node for Mode A.

| Component | Pin |
|-----------|-----|
| `reth-*` / `reth-ethereum` | `git = "https://github.com/op-rs/reth"` · `rev = "aef8d3ef92117f91455e16969f0adf5bf7c6e9e1"` |
| `reth-optimism-*` | `git = "https://github.com/ethereum-optimism/optimism"` · `tag = "op-reth/v2.4.4"` |
| `op-node` (binary; not Cargo) | **v1.19.7** (minimum v1.19.1) |

That `op-rs/reth` rev is exactly what `op-reth/v2.4.4` uses. Do not pick a newer commit. Do not float `main`.

## Rules

1. **One Reth remote.** If `Cargo.lock` contains both `paradigmxyz/reth` and `op-rs/reth`, stop and fix.
2. Do not fork Reth. Do not override `revm` past the pin.
3. Add `reth-optimism-*` crates only as the compiler asks; do not import the whole optimism workspace.
4. Local Mode A genesis / `rollup.json` / L1 portal come from **op-deployer** (or the official local-dev path for op-node/v1.19.7). Do not invent portal addresses.

## Why

`paradigmxyz/reth` `v2.5.2` has no `reth-optimism-*` crates (op-reth moved to `ethereum-optimism/optimism`). Mode A needs those crates plus a matching Reth. ADR-016 chooses option 2 from the Sprint 2 conflict: explicitly change the pin rather than wait or stay on `--dev` only.

## Consequences

- `--dev` and `--engine` both compile against `op-rs/reth@aef8d3ef…`.
- Alloy on Reth-facing crates follows that pin (~`alloy-evm` 0.37.x), not a newer float.
- Mode A: `quub-node --engine` (9545 / authrpc 9551) + `op-node` v1.19.7. Chain id remains **8091**.
