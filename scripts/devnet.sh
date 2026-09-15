#!/usr/bin/env bash
# Sprint 1: start quub-node (programmatic --dev HTTP) and call F202.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

RPC="http://127.0.0.1:8545"

cleanup() {
  if [[ -n "${NODE_PID:-}" ]] && kill -0 "$NODE_PID" 2>/dev/null; then
    kill "$NODE_PID" 2>/dev/null || true
    wait "$NODE_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# Free HTTP / leftover auth so cast probes the node we start.
if command -v lsof >/dev/null 2>&1; then
  lsof -tiTCP:8545 -sTCP:LISTEN 2>/dev/null | xargs kill 2>/dev/null || true
  lsof -tiTCP:8551 -sTCP:LISTEN 2>/dev/null | xargs kill 2>/dev/null || true
fi
pkill -f 'target/debug/quub-node' 2>/dev/null || true
sleep 1

cargo run -p quub-node &
NODE_PID=$!

ready=0
for i in $(seq 1 90); do
  if cast chain-id --rpc-url "$RPC" >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 2
done
if [[ "$ready" -ne 1 ]]; then
  echo "RPC did not become ready on $RPC" >&2
  exit 1
fi

CHAIN_ID="$(cast chain-id --rpc-url "$RPC")"
echo "chain-id: $CHAIN_ID"
if [[ "$CHAIN_ID" != "8091" ]]; then
  echo "expected chain id 8091" >&2
  exit 1
fi

HASH="$(cast call 0x000000000000000000000000000000000000F202 \
  "validateAndCommit(bytes32,bytes16,bytes32,bytes3,uint8)(bytes32)" \
  0x1111111111111111111111111111111111111111111111111111111111111111 \
  0x22222222222222222222222222222222 \
  0x3333333333333333333333333333333333333333333333333333333333333333 \
  0x555344 \
  0 \
  --rpc-url "$RPC")"

echo "F202 memoHash: $HASH"
if [[ "$HASH" == "0x" || "$HASH" == "0x0000000000000000000000000000000000000000000000000000000000000000" ]]; then
  echo "F202 did not return a hash (precompile not registered?)" >&2
  exit 1
fi
