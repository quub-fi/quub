#!/usr/bin/env bash
# Sprint 2 Mode A: geth L1 + op-node + quub-node --engine (ADR-016).
# Requires: cast, geth (or GETH_BIN), cargo, artifacts/mode-a/{bin/op-node,jwt.txt,deployer/*}.
# Do NOT use anvil as the L1 that op-node derives from (empty state root breaks SystemConfig proofs).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ART="$ROOT/artifacts/mode-a"
DEPLOYER="$ART/deployer"
OP_NODE_BIN="${OP_NODE_BIN:-$ART/bin/op-node}"
GETH_BIN="${GETH_BIN:-$ART/bin/geth}"
JWT="$ART/jwt.txt"
L2_RPC="http://127.0.0.1:9545"
L1_RPC="http://127.0.0.1:8546"
AUTH_RPC="http://127.0.0.1:9551"
GENESIS_L2="$DEPLOYER/l2-genesis-quub.json"
ROLLUP="$DEPLOYER/rollup.json"
L1_GENESIS="$DEPLOYER/l1-genesis-full.json"
L1_DATADIR="${L1_DATADIR:-$ART/l1-geth}"
# Default reth platform path for chain 8091 (quub-node --engine has no --datadir yet)
RETH_DATADIR="${RETH_DATADIR:-$HOME/Library/Application Support/reth/8091}"

DEV_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
DEV_ADDR="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
F210="0x000000000000000000000000000000000000F210"
F211="0x000000000000000000000000000000000000F211"
F213="0x000000000000000000000000000000000000F213"
SYSCONF="0xe991d1A48616a1F28d26476F53F103731fEFC990"
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
  pkill -f 'geth.*mode-a/l1-geth' 2>/dev/null || true
}
trap cleanup EXIT

die() { echo "mode-a: $*" >&2; exit 1; }

[[ -x "$OP_NODE_BIN" ]] || die "missing op-node at $OP_NODE_BIN (build v1.19.7 into artifacts/mode-a/bin/)"
[[ -f "$GENESIS_L2" ]] || die "missing $GENESIS_L2"
[[ -f "$ROLLUP" ]] || die "missing $ROLLUP (op-deployer inspect)"
[[ -f "$L1_GENESIS" ]] || die "missing $L1_GENESIS (op-deployer L1 genesis)"
if [[ ! -x "$GETH_BIN" ]]; then
  if command -v geth >/dev/null; then
    GETH_BIN="$(command -v geth)"
  else
    die "missing geth (set GETH_BIN or install into $ART/bin/geth)"
  fi
fi
command -v cast >/dev/null || die "cast required"

# JWT (32 bytes hex)
if [[ ! -f "$JWT" ]]; then
  openssl rand -hex 32 >"$JWT"
fi

# Free ports + wipe L2 datadir so genesis overlay is fresh
for port in 8546 9545 9551 30303 30304; do
  lsof -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null | xargs kill 2>/dev/null || true
done
pkill -f 'quub-node --engine' 2>/dev/null || true
pkill -f 'geth.*mode-a/l1-geth' 2>/dev/null || true
pkill -f 'artifacts/mode-a/bin/op-node' 2>/dev/null || true
rm -rf "$RETH_DATADIR"
sleep 1

echo "=== Mode A (ADR-016) ==="
echo "op-node: $($OP_NODE_BIN --version 2>&1 | head -1)"
echo "geth: $($GETH_BIN version 2>&1 | head -1)"
echo "L1 RPC $L1_RPC | L2 HTTP $L2_RPC | authrpc $AUTH_RPC | chain 8091"
echo "SystemConfig: $SYSCONF"

# 1) L1 — geth with full deployer L1 genesis (preserves SystemConfig state root)
rm -rf "$L1_DATADIR"
mkdir -p "$L1_DATADIR"
"$GETH_BIN" --datadir "$L1_DATADIR" init "$L1_GENESIS" >/tmp/quub-mode-a-geth-init.log 2>&1 \
  || die "geth init failed — see /tmp/quub-mode-a-geth-init.log"

# Post-merge L1 (TTD=0). No beacon needed for SystemConfig proofs at genesis;
# op-node --l1.beacon.ignore sequences L2 against L1 origin 0.
"$GETH_BIN" \
  --datadir "$L1_DATADIR" \
  --networkid 1 \
  --http --http.addr 127.0.0.1 --http.port 8546 \
  --http.api eth,net,web3,debug \
  --http.corsdomain '*' \
  --http.vhosts '*' \
  --nodiscover --maxpeers 0 \
  --syncmode full \
  --gcmode archive \
  --port 30304 \
  >/tmp/quub-mode-a-geth.log 2>&1 &
GETH_PID=$!
PIDS+=("$GETH_PID")

for i in $(seq 1 90); do
  cast chain-id --rpc-url "$L1_RPC" >/dev/null 2>&1 && break
  if ! kill -0 "$GETH_PID" 2>/dev/null; then
    echo "geth died:" >&2
    cat /tmp/quub-mode-a-geth.log >&2
    exit 1
  fi
  sleep 1
done
L1_ID="$(cast chain-id --rpc-url "$L1_RPC")"
[[ "$L1_ID" == "1" ]] || die "L1 chain-id=$L1_ID expected 1"

L1_STATE="$(cast block 0 --rpc-url "$L1_RPC" --json | python3 -c 'import json,sys; d=json.load(sys.stdin); d=d.get("data",d); print(d["stateRoot"])')"
L1_HASH="$(cast block 0 --rpc-url "$L1_RPC" --json | python3 -c 'import json,sys; d=json.load(sys.stdin); d=d.get("data",d); print(d["hash"])')"
echo "L1 genesis hash: $L1_HASH stateRoot: $L1_STATE"
[[ "$L1_STATE" != "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421" ]] \
  || die "L1 state root is empty trie — genesis alloc not loaded"

SC_CODE="$(cast codesize "$SYSCONF" --rpc-url "$L1_RPC")"
[[ "$SC_CODE" != "0" ]] || die "SystemConfig $SYSCONF codesize 0 on L1"
echo "SystemConfig codesize: $SC_CODE"

python3 - <<PY
import json
from pathlib import Path
r = json.loads(Path("$ROLLUP").read_text())
sealed = r["genesis"]["l1"]["hash"]
live = "$L1_HASH"
if sealed.lower() != live.lower():
    print(f"mode-a: WARN L1 hash sealed={sealed} live={live} — patching rollup.runtime.json")
    r["genesis"]["l1"]["hash"] = live
    r["genesis"]["l1"]["number"] = 0
else:
    print("mode-a: L1 genesis hash matches sealed rollup.json")
Path("$ART/rollup.runtime.json").write_text(json.dumps(r, indent=2) + "\n")
print("wrote $ART/rollup.runtime.json")
PY

# 2) quub-node --engine
export PATH="$HOME/.rustup/toolchains/1.96.0-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"
export CARGO_HOME="${CARGO_HOME:-/tmp/quub-cargo-home-adr016}"
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
SENDER_CS="$(cast codesize "$DEV_ADDR" --rpc-url "$L2_RPC")"
[[ "$SENDER_CS" == "0" ]] || die "dev sender codesize=$SENDER_CS (expected EOA)"

L2_HASH="$(cast block 0 --rpc-url "$L2_RPC" --json | python3 -c 'import json,sys; d=json.load(sys.stdin); d=d.get("data",d); print(d["hash"])')"
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

bn=0
for i in $(seq 1 180); do
  bn="$(cast block-number --rpc-url "$L2_RPC" 2>/dev/null || echo 0)"
  if [[ "$bn" -ge 1 ]]; then
    break
  fi
  if ! kill -0 "$OP_PID" 2>/dev/null; then
    echo "op-node died:" >&2
    tail -60 /tmp/quub-mode-a-opnode.log >&2
    exit 1
  fi
  sleep 2
done
echo "L2 block-number: $bn"
[[ "$bn" -ge 1 ]] || {
  echo "op-node log (tail):" >&2
  tail -60 /tmp/quub-mode-a-opnode.log >&2
  die "L2 did not advance (op-node / Engine API)"
}

# 4) transferWithMemo on 9545 (EIP-1559; avoid legacy --gas-price on OP)
set +e
PAY_OUT="$(cast send "$F210" \
  "transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)" \
  "$TO" 1000000 "$E2E" "$UETR" "$INSTR" "$CCY" 0 "$TR_HASH" "$PACK_HASH" \
  --rpc-url "$L2_RPC" \
  --private-key "$DEV_KEY" \
  --gas-limit 500000 2>/tmp/quub-mode-a-pay.err)"
PAY_EC=$?
set -e
[[ "$PAY_EC" -eq 0 ]] || {
  cat /tmp/quub-mode-a-pay.err >&2
  die "transferWithMemo cast send failed"
}
# Prefer table line; fall back to last 32-byte hash in output (receipt tx hash).
TX="$(printf '%s\n' "$PAY_OUT" | awk '/^transactionHash[[:space:]]+/{print $2; exit}')"
if [[ -z "$TX" ]]; then
  TX="$(printf '%s\n' "$PAY_OUT" | python3 -c 'import re,sys; m=re.findall(r"0x[a-fA-F0-9]{64}", sys.stdin.read()); print(m[-1] if m else "")')"
fi
echo "L2 transferWithMemo: $TX"
[[ -n "$TX" && "$TX" =~ ^0x[a-fA-F0-9]{64}$ ]] || die "bad tx hash — cast output: $PAY_OUT"

STATUS="$(cast receipt "$TX" --rpc-url "$L2_RPC" --json | python3 -c 'import json,sys; d=json.load(sys.stdin); d=d.get("data",d) if isinstance(d,dict) else d; print(d["status"] if isinstance(d,dict) else d)')"
echo "receipt status: $STATUS"
[[ "$STATUS" == "0x1" || "$STATUS" == "1" ]] || die "payment failed"

# 5) freeze story — cast send exits 0 even when the mined tx reverts (status 0x0)
cast send "$F211" "freeze(address)" "$DEV_ADDR" \
  --rpc-url "$L2_RPC" --private-key "$DEV_KEY" --gas-limit 200000 >/dev/null
set +e
FREEZE_OUT="$(cast send "$F210" \
  "transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)" \
  "$TO" 1 "$E2E" "$UETR" "$INSTR" "$CCY" 0 "$TR_HASH" "$PACK_HASH" \
  --rpc-url "$L2_RPC" --private-key "$DEV_KEY" \
  --gas-limit 500000 2>/tmp/quub-mode-a-freeze.err)"
FREEZE_EC=$?
set -e
FREEZE_TX="$(printf '%s\n' "$FREEZE_OUT" | awk '/^transactionHash[[:space:]]+/{print $2; exit}')"
if [[ -z "$FREEZE_TX" ]]; then
  FREEZE_TX="$(printf '%s\n' "$FREEZE_OUT" | python3 -c 'import re,sys; m=re.findall(r"0x[a-fA-F0-9]{64}", sys.stdin.read()); print(m[-1] if m else "")')"
fi
if [[ -n "$FREEZE_TX" ]]; then
  FZ_STATUS="$(cast receipt "$FREEZE_TX" --rpc-url "$L2_RPC" --json | python3 -c 'import json,sys; d=json.load(sys.stdin); d=d.get("data",d) if isinstance(d,dict) else d; print(d["status"] if isinstance(d,dict) else d)')"
  [[ "$FZ_STATUS" == "0x0" || "$FZ_STATUS" == "0" ]] || die "expected freeze transfer to revert, status=$FZ_STATUS"
else
  # RPC-level rejection also counts as freeze working
  [[ "$FREEZE_EC" -ne 0 ]] || die "expected freeze to revert (no tx hash, cast ok)"
fi
echo "freeze: second send reverted (ok)"
cast send "$F211" "unfreeze(address)" "$DEV_ADDR" \
  --rpc-url "$L2_RPC" --private-key "$DEV_KEY" --gas-limit 200000 >/dev/null

# Kill-test: stop op-node, L2 must not keep mining
kill "$OP_PID" 2>/dev/null || true
wait "$OP_PID" 2>/dev/null || true
PIDS=("${PIDS[@]/$OP_PID}")
BN1="$(cast block-number --rpc-url "$L2_RPC")"
sleep 6
BN2="$(cast block-number --rpc-url "$L2_RPC")"
echo "kill-test: block $BN1 -> $BN2 (expect unchanged)"
[[ "$BN1" == "$BN2" ]] || die "L2 kept mining without op-node (--dev miner leak)"

echo "=== Mode A OK ==="
echo "deployer: see artifacts/mode-a/DEPLOYER.txt"
echo "L1 client: geth ($GETH_BIN)"
echo "SystemConfig: $SYSCONF"
echo "op-node: $($OP_NODE_BIN --version 2>&1 | head -1)"
echo "9545 chain-id: $L2_ID"
echo "L2 payment: $TX"
echo "freeze: reverted (ok)"
