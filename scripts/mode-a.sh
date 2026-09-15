#!/usr/bin/env bash
# Sprint 2 Mode A: local L1 + op-node + quub-node --engine.
# Currently exits with the Reth / op-reth version conflict (do not bump Reth).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cat >&2 <<'EOF'
Mode A blocked (Sprint 2 stop condition).

Quub Reth pin: paradigmxyz/reth tag v2.5.2 — no crates/optimism / reth-optimism-*.
op-reth lives in ethereum-optimism/optimism and pins op-rs/reth (latest checked: op-reth/v2.4.4),
not paradigmxyz/reth@v2.5.2.

See crates/quub-consensus-op/CONFLICT.md and README "Mode A version conflict".
Do not bump Reth in this sprint.

--dev path remains: bash scripts/devnet.sh (HTTP 8545).
When unblocked: quub-node --engine HTTP 9545 / authrpc 9551, chain id 8091, same F210–F213 alloc.
EOF

exit 1
