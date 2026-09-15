# Mode A / op-reth pin (Sprint 2)

**Status:** **resolved by [ADR-016](../../../docs/adr/ADR-016-execution-pin.md).**

Quub now pins:

| Component | Pin |
|-----------|-----|
| `reth-*` | `https://github.com/op-rs/reth` @ `aef8d3ef92117f91455e16969f0adf5bf7c6e9e1` |
| `reth-optimism-*` | `https://github.com/ethereum-optimism/optimism` @ tag `op-reth/v2.4.4` |
| `op-node` | v1.19.7 (binary) |

One Reth remote only. Mode A launch lives in this crate once `--dev` is green on the new pin.

---

## History (pre–ADR-016)

**Was:** blocked. Do not bump Reth. Do not enable Mode A launch.

### Locked Quub pin (old)

```toml
reth-ethereum = { git = "https://github.com/paradigmxyz/reth", tag = "v2.5.2", ... }
```

At `paradigmxyz/reth` tag **`v2.5.2`**:

- There is **no** `crates/optimism/` directory.
- There are **no** publishable `reth-optimism-node`, `reth-optimism-evm`, or related crates in that tree.
- Upstream README: OP-Reth moved to `ethereum-optimism/optimism`.

### Where op-reth lived

| Item | Value |
|------|--------|
| Repo | `ethereum-optimism/optimism` (`rust/op-reth/`) |
| Latest tag checked | `op-reth/v2.4.4` |
| Reth git source | `https://github.com/op-rs/reth` (not `paradigmxyz/reth@v2.5.2`) |

### Why Mode A could not wire on the old pin

OP deposit types and Engine API builders live in `reth-optimism-*`, which do not compile as dependencies of `paradigmxyz/reth@v2.5.2`. Pulling optimism’s crates pulls `op-rs/reth`, which violated “do not silently bump Reth” / one pin.

### Options then (human chose #2 → ADR-016)

1. Wait until optimism tracks `paradigmxyz/reth` `v2.5.2`, or
2. **Explicitly approve a Reth pin change** (ADR-016), or
3. Stay on `--dev` only.
