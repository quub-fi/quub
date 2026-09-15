#!/usr/bin/env bash
# Sprint 2 Mode A: local L1 + op-node + quub-node --engine (ADR-016).
# Requires: cast, anvil, cargo, artifacts/mode-a/{bin/op-node,jwt.txt,deployer/*}.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ART="$ROOT/artifacts/mode-a"
DEPLOYER="$ART/deployer"
OP_NODE_BIN="${OP_NODE_BIN:-$ART/bin/op-node}"
JWT="$ART/jwt.txt"
L2_RPC="http://127.0.0.1:9545"
L1_RPC="http://127.0.0.1:8546"
AUTH_RPC="http://127.0.0.1:9551"
GENESIS_L2="$DEPLOYER/l2-genesis-quub.json"
ROLLUP="$DEPLOYER/rollup.json"
L1_GENESIS="$DEPLOYER/l1-genesis-full.json"

DEV_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
DEV_ADDR="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
F210="0x000000000000000000000000000000000000F210"
F211="0x000000000000000000000000000000000000F211"
F213="0x000000000000000000000000000000000000F213"
TO="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
E2E="0x1111111111111111111111111111111111111111111111111111111111111111"
UETR="0x22222222222222222222222222222222"
INSTR="0x3333333333333333333333333333333333333333333333333333333333333333"
CCY="0x555344"
TR_HASH="0x0000000000000000000000000000000000000000000000000000000000000000"
PACK_HASH="0x4444444444444444444444444444444444444444444444444444444444444444"

PIDS=()
cleanup() {
  for p in "${PIDS[@]:-}"; do
    kill "$p" 2>/dev/null || true
    wait "$p" 2>/dev/null || true
  done
  pkill -f 'quub-node --engine' 2>/dev/null || true
}
trap cleanup EXIT

die() { echo "mode-a: $*" >&2; exit 1; }

[[ -x "$OP_NODE_BIN" ]] || die "missing op-node at $OP_NODE_BIN (build v1.19.7 into artifacts/mode-a/bin/)"
[[ -f "$GENESIS_L2" ]] || die "missing $GENESIS_L2"
[[ -f "$ROLLUP" ]] || die "missing $ROLLUP (op-deployer inspect)"
command -v anvil >/dev/null || die "anvil (Foundry) required for local L1"
command -v cast >/dev/null || die "cast required"

# JWT (32 bytes hex)
if [[ ! -f "$JWT" ]]; then
  openssl rand -hex 32 >"$JWT"
fi

# Free ports
for port in 8546 9545 9551 30303; do
  lsof -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null | xargs kill 2>/dev/null || true
done
pkill -f 'quub-node --engine' 2>/dev/null || true
sleep 1

echo "=== Mode A (ADR-016) ==="
echo "op-node: $($OP_NODE_BIN --version 2>&1 | head -1)"
echo "L1 RPC $L1_RPC | L2 HTTP $L2_RPC | authrpc $AUTH_RPC | chain 8091"

# 1) L1 — anvil with deployer L1 genesis (chain id 1 in rollup.json)
anvil --port 8546 --chain-id 1 --init "$L1_GENESIS" --silent >/tmp/quub-mode-a-anvil.log 2>&1 &
PIDS+=($!)
for i in $(seq 1 60); do
  cast chain-id --rpc-url "$L1_RPC" >/dev/null 2>&1 && break
  sleep 1
done
L1_ID="$(cast chain-id --rpc-url "$L1_RPC")"
[[ "$L1_ID" == "1" ]] || die "L1 chain-id=$L1_ID expected 1"
L1_HASH="$(cast block 0 --rpc-url "$L1_RPC" --json | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("data",d)["hash"])')"
echo "L1 genesis hash: $L1_HASH"

# Patch rollup.json L1 genesis hash to match anvil block 0 (op-deployer hash may differ under anvil).
python3 - <<PY
import json
from pathlib import Path
p = Path("$ROLLUP")
r = json.loads(p.read_text())
r["genesis"]["l1"]["hash"] = "$L1_HASH"
r["genesis"]["l1"]["number"] = 0
# L2 genesis hash must match quub overlay genesis — compute via cast if available later.
Path("$ART/rollup.runtime.json").write_text(json.dumps(r, indent=2) + "\n")
print("wrote $ART/rollup.runtime.json")
PY

# 2) quub-node --engine
cargo build -p quub-node -q
./target/debug/quub-node --engine \
  --http-port 9545 \
  --auth-port 9551 \
  --jwt "$JWT" \
  --genesis "$GENESIS_L2" \
  >/tmp/quub-mode-a-engine.log 2>&1 &
ENGINE_PID=$!
PIDS+=("$ENGINE_PID")

ready=0
for i in $(seq 1 120); do
  if cast chain-id --rpc-url "$L2_RPC" >/dev/null 2>&1; then
    ready=1
    break
  fi
  if ! kill -0 "$ENGINE_PID" 2>/dev/null; then
    echo "quub-node --engine died:" >&2
    cat /tmp/quub-mode-a-engine.log >&2
    exit 1
  fi
  sleep 1
done
[[ "$ready" -eq 1 ]] || die "L2 RPC not up on $L2_RPC"
L2_ID="$(cast chain-id --rpc-url "$L2_RPC")"
[[ "$L2_ID" == "8091" ]] || die "L2 chain-id=$L2_ID expected 8091"
echo "L2 chain-id: $L2_ID"

CS="$(cast codesize "$F213" --rpc-url "$L2_RPC")"
[[ "$CS" != "0" ]] || die "F213 codesize 0 — Quub alloc missing from L2 genesis"
echo "F213 codesize: $CS"

L2_HASH="$(cast block 0 --rpc-url "$L2_RPC" --json | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("data",d)["hash"])')"
echo "L2 genesis hash: $L2_HASH"
python3 - <<PY
import json
from pathlib import Path
p = Path("$ART/rollup.runtime.json")
r = json.loads(p.read_text())
r["genesis"]["l2"]["hash"] = "$L2_HASH"
r["genesis"]["l2"]["number"] = 0
p.write_text(json.dumps(r, indent=2) + "\n")
print("updated rollup.runtime.json L2 genesis hash")
PY

# 3) op-node (sequencer)
"$OP_NODE_BIN" \
  --l1="$L1_RPC" \
  --l2="$AUTH_RPC" \
  --l2.jwt-secret="$JWT" \
  --rollup.config="$ART/rollup.runtime.json" \
  --sequencer.enabled \
  --sequencer.l1-confs=0 \
  --verifier.l1-confs=0 \
  --l1.beacon.ignore \
  --l1.beacon.slot-duration-override=12 \
  --p2p.disable \
  --rpc.addr=127.0.0.1 \
  --rpc.port=9546 \
  >/tmp/quub-mode-a-opnode.log 2>&1 &
OP_PID=$!
PIDS+=("$OP_PID")

# Wait for L2 block >= 1
bn=0
for i in $(seq 1 120); do
  bn="$(cast block-number --rpc-url "$L2_RPC" 2>/dev/null || echo 0)"
  if [[ "$bn" -ge 1 ]]; then
    break
  fi
  sleep 2
done
echo "L2 block-number: $bn"
[[ "$bn" -ge 1 ]] || {
  echo "op-node log (tail):" >&2
  tail -40 /tmp/quub-mode-a-opnode.log >&2
  die "L2 did not advance (op-node / Engine API)"
}

# 4) transferWithMemo on 9545
TX="$(cast send "$F210" \
  "transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)" \
  "$TO" 1000000 "$E2E" "$UETR" "$INSTR" "$CCY" 0 "$TR_HASH" "$PACK_HASH" \
  --rpc-url "$L2_RPC" \
  --private-key "$DEV_KEY" \
  --legacy --gas-price 1000000000 --gas-limit 500000 \
  --json | python3 -c 'import json,sys; d=json.load(sys.stdin); d=d.get("data",d); print(d.get("transactionHash") or d.get("hash") or "")')"
echo "L2 transferWithMemo: $TX"
[[ -n "$TX" ]] || die "empty tx hash"

STATUS="$(cast receipt "$TX" --rpc-url "$L2_RPC" --json | python3 -c 'import json,sys; d=json.load(sys.stdin); d=d.get("data",d); print(d["status"])')"
echo "receipt status: $STATUS"
[[ "$STATUS" == "0x1" || "$STATUS" == "1" ]] || die "payment failed"

# 5) freeze story
cast send "$F211" "freeze(address)" "$DEV_ADDR" \
  --rpc-url "$L2_RPC" --private-key "$DEV_KEY" --legacy --gas-price 1000000000 >/dev/null
if cast send "$F210" \
  "transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)" \
  "$TO" 1 "$E2E" "$UETR" "$INSTR" "$CCY" 0 "$TR_HASH" "$PACK_HASH" \
  --rpc-url "$L2_RPC" --private-key "$DEV_KEY" \
  --legacy --gas-price 1000000000 --gas-limit 500000 2>/tmp/quub-mode-a-freeze.err; then
  die "expected freeze to revert"
fi
echo "freeze: second send reverted (ok)"
cast send "$F211" "unfreeze(address)" "$DEV_ADDR" \
  --rpc-url "$L2_RPC" --private-key "$DEV_KEY" --legacy --gas-price 1000000000 >/dev/null

# Kill-test: stop op-node, L2 must not keep mining
kill "$OP_PID" 2>/dev/null || true
wait "$OP_PID" 2>/dev/null || true
BN1="$(cast block-number --rpc-url "$L2_RPC")"
sleep 6
BN2="$(cast block-number --rpc-url "$L2_RPC")"
echo "kill-test: block $BN1 -> $BN2 (expect unchanged)"
[[ "$BN1" == "$BN2" ]] || die "L2 kept mining without op-node (--dev miner leak)"

echo "=== Mode A OK ==="
echo "L2 payment: $TX"
echo "ps during run included quub-node --engine + op-node (no Simplex)"
