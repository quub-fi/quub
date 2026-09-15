# Mode A / op-reth conflict (Sprint 2)

**Status:** blocked. Do not bump Reth. Do not enable Mode A launch.

## Locked Quub pin

```toml
reth-ethereum = { git = "https://github.com/paradigmxyz/reth", tag = "v2.5.2", ... }
```

At `paradigmxyz/reth` tag **`v2.5.2`**:

- There is **no** `crates/optimism/` directory.
- There are **no** publishable `reth-optimism-node`, `reth-optimism-evm`, or related crates in that tree.
- Upstream README: OP-Reth moved to `ethereum-optimism/optimism`.

## Where op-reth lives now

| Item | Value |
|------|--------|
| Repo | `ethereum-optimism/optimism` (`rust/op-reth/`) |
| Latest tag checked | `op-reth/v2.4.4` |
| Reth git source used by that tag line / `develop` | `https://github.com/op-rs/reth` (fork / mirror), **not** `paradigmxyz/reth@v2.5.2` |

## Why Quub cannot wire Engine API + OP deposits on this pin

Sprint 2 requires:

- OP deposit transaction types accepted by the executor
- Engine API surface for `op-node`
- Same `QuubEvmFactory` / F201–F203

Those types and node builders live in `reth-optimism-*` (now under optimism’s `op-reth` crates), which **do not compile as dependencies of `paradigmxyz/reth@v2.5.2`**. Pulling `ethereum-optimism/optimism`’s crates pulls `op-rs/reth`, which violates “do not silently bump Reth” / one pin.

## Allowed next steps (human decision; not this sprint)

1. Wait until optimism publishes an op-reth line that tracks `paradigmxyz/reth` **`v2.5.2`** (or a later 2.5.x Quub is allowed to take), **or**
2. Explicitly approve a Reth pin change in AGENTS.md / Cargo.toml, **or**
3. Stay on `--dev` only until then.

## What this crate does meanwhile

`quub-consensus-op` remains a compile-only placeholder. It must not start `op-node` or claim Engine API readiness.
