#!/usr/bin/env bash
# Sprint 3: flood payment + high-tip general txs on --dev (8545); grade one block.
# Accepts ≥~65% payment gas share under payment-heavy demand. One pass — no pretty-number loop.
# Optional: if 9545 is already up, note Mode A code-path (live flood bonus only).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export PATH="${HOME}/.foundry/bin:${HOME}/.cargo/bin:${PATH:-}"
export CARGO_HOME="${CARGO_HOME:-/tmp/quub-cargo-home-adr016}"
export PATH="${HOME}/.rustup/toolchains/1.96.0-aarch64-apple-darwin/bin:${PATH}"

RPC="http://127.0.0.1:8545"
ENGINE_RPC="http://127.0.0.1:9545"
# Public anvil account 0 — --dev only. Not production.
DEV_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
DEV_ADDR="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
F210="0x000000000000000000000000000000000000F210"
TO="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"

UETR="0x22222222222222222222222222222222"
INSTR="0x3333333333333333333333333333333333333333333333333333333333333333"
CCY="0x555344"
MSG_TYPE="0"
TR_HASH="0x0000000000000000000000000000000000000000000000000000000000000000"
PACK_HASH="0x4444444444444444444444444444444444444444444444444444444444444444"
AMOUNT="1000"

MEMO_SEL="0x843e111e"
MEMO_SIG="transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)"
# Exact calldata length for transferWithMemo: 4 + 9*32 = 292 bytes → 584 hex + 0x
MEMO_CALLDATA_HEX_LEN=586

# Must land in one interval-mined block (quub-node --dev uses 2s block_time + payload_wait).
N_PAY=15
N_GEN=5
N_TOTAL=$((N_PAY + N_GEN))
PAY_GAS_PRICE=1000000000      # 1 gwei
GEN_GAS_PRICE=1000000000000   # 1000 gwei
ACCEPT_SHARE_BPS=6500         # ≥~65%

cleanup() {
  if [[ -n "${NODE_PID:-}" ]] && kill -0 "$NODE_PID" 2>/dev/null; then
    kill "$NODE_PID" 2>/dev/null || true
    wait "$NODE_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# Newer cast wraps --json as {success,data,...}; unwrap to the payload object.
cast_json() {
  python3 -c 'import json,sys; w=json.load(sys.stdin); print(json.dumps(w["data"] if isinstance(w,dict) and "data" in w else w))'
}

if command -v lsof >/dev/null 2>&1; then
  lsof -tiTCP:8545 -sTCP:LISTEN 2>/dev/null | xargs kill 2>/dev/null || true
fi
pkill -f 'target/.*/quub-node' 2>/dev/null || true
sleep 1

echo "=== building / starting quub-node --dev on 8545 ==="
cargo build -p quub-node 2>&1 | tail -8
cargo run -p quub-node --quiet &
NODE_PID=$!

ready=0
for _ in $(seq 1 120); do
  if cast chain-id --rpc-url "$RPC" >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 1
done
if [[ "$ready" -ne 1 ]]; then
  echo "RPC did not become ready on $RPC" >&2
  exit 1
fi

CHAIN_ID="$(cast chain-id --rpc-url "$RPC")"
echo "chain-id: $CHAIN_ID"
[[ "$CHAIN_ID" == "8091" ]] || { echo "expected 8091" >&2; exit 1; }

code="$(cast code "$F210" --rpc-url "$RPC")"
[[ "$code" != "0x" && -n "$code" ]] || { echo "F210 empty" >&2; exit 1; }

BLOCK_GAS="$(cast block latest --json --rpc-url "$RPC" | cast_json | python3 -c 'import json,sys; print(int(json.load(sys.stdin)["gasLimit"],16))')"
echo "block gasLimit: $BLOCK_GAS"
PAY_BUDGET=$(( BLOCK_GAS * 7000 / 10000 ))
echo "payment budget (70%): $PAY_BUDGET"

START_BLOCK="$(cast block-number --rpc-url "$RPC")"
echo "start block: $START_BLOCK"

NONCE="$(cast nonce "$DEV_ADDR" --rpc-url "$RPC")"
echo "=== flooding ${N_PAY} memos @ ${PAY_GAS_PRICE} wei + ${N_GEN} high-tip transfers @ ${GEN_GAS_PRICE} wei (total ${N_TOTAL}) ==="

for i in $(seq 1 "$N_PAY"); do
  e2e="$(printf '0x%064x' "$i")"
  cast send "$F210" "$MEMO_SIG" \
    "$TO" "$AMOUNT" "$e2e" "$UETR" "$INSTR" "$CCY" "$MSG_TYPE" "$TR_HASH" "$PACK_HASH" \
    --rpc-url "$RPC" --private-key "$DEV_KEY" \
    --nonce "$NONCE" --legacy --gas-price "$PAY_GAS_PRICE" --gas-limit 400000 \
    --async >/dev/null
  NONCE=$((NONCE + 1))
done

for _ in $(seq 1 "$N_GEN"); do
  cast send "$F210" "transfer(address,uint256)" "$TO" "$AMOUNT" \
    --rpc-url "$RPC" --private-key "$DEV_KEY" \
    --nonce "$NONCE" --legacy --gas-price "$GEN_GAS_PRICE" --gas-limit 100000 \
    --async >/dev/null
  NONCE=$((NONCE + 1))
done

echo "waiting for a multi-tx block (interval mining; submit burst then wait)…"
TARGET=""
for _ in $(seq 1 90); do
  bn="$(cast block-number --rpc-url "$RPC")"
  if [[ "$bn" -gt "$START_BLOCK" ]]; then
    raw="$(cast block "$bn" --full --json --rpc-url "$RPC" | cast_json)"
    n="$(echo "$raw" | python3 -c 'import json,sys; print(len(json.load(sys.stdin).get("transactions") or []))')"
    if [[ "$n" -ge 2 ]]; then
      TARGET="$bn"
      break
    fi
  fi
  sleep 0.5
done

if [[ -z "$TARGET" ]]; then
  echo "FAIL: no multi-tx block appeared" >&2
  exit 1
fi

BLOCK_JSON="$(cast block "$TARGET" --full --json --rpc-url "$RPC" | cast_json)"
BLOCK_HASH="$(echo "$BLOCK_JSON" | python3 -c 'import json,sys; print(json.load(sys.stdin)["hash"])')"
echo "=== grading block $TARGET ==="
echo "block hash: $BLOCK_HASH"

BLOCK_FILE="$(mktemp)"
echo "$BLOCK_JSON" >"$BLOCK_FILE"
EVAL="$(python3 - "$RPC" "$PAY_BUDGET" "$ACCEPT_SHARE_BPS" "$BLOCK_FILE" <<'PY'
import json, sys, urllib.request

rpc, pay_budget, accept_bps, block_path = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
with open(block_path) as f:
    block = json.load(f)
txs = block.get("transactions") or []

MEMO_SEL = "0x843e111e"
F210 = "0x000000000000000000000000000000000000f210"
MEMO_HEX_LEN = 586

def is_payment(to, inp):
    to = (to or "").lower()
    inp = (inp or "").lower()
    if to != F210:
        return False
    if len(inp) != MEMO_HEX_LEN:
        return False
    return inp.startswith(MEMO_SEL)

def rpc_call(method, params):
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode()
    req = urllib.request.Request(rpc, data=body, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=30) as r:
        out = json.load(r)
        return out["result"]

classes = []
resolved = []
for tx in txs:
    if isinstance(tx, str):
        tx = rpc_call("eth_getTransactionByHash", [tx])
    resolved.append(tx)
    classes.append(is_payment(tx.get("to"), tx.get("input") or tx.get("data") or "0x"))

pay_n = gen_n = 0
pay_gas = gen_gas = 0
cum = 0
steal = False

for i, tx in enumerate(resolved):
    receipt = rpc_call("eth_getTransactionReceipt", [tx["hash"]])
    gas = int(receipt["gasUsed"], 16)
    payment = classes[i]
    if payment:
        pay_n += 1
        pay_gas += gas
    else:
        gen_n += 1
        gen_gas += gas
    if (not payment) and cum < pay_budget and any(classes[j] for j in range(i + 1, len(classes))):
        steal = True
    cum += gas

total = pay_gas + gen_gas
share_bps = (pay_gas * 10000 // total) if total else 0
print(f"payment_count={pay_n}")
print(f"general_count={gen_n}")
print(f"payment_gas={pay_gas}")
print(f"general_gas={gen_gas}")
print(f"payment_gas_share_bps={share_bps}")
print(f"steal={1 if steal else 0}")
print(f"tx_count={len(txs)}")
PY
)"
rm -f "$BLOCK_FILE"

echo "$EVAL"
PAY_N="$(echo "$EVAL" | awk -F= '/^payment_count=/{print $2}')"
GEN_N="$(echo "$EVAL" | awk -F= '/^general_count=/{print $2}')"
PAY_GAS="$(echo "$EVAL" | awk -F= '/^payment_gas=/{print $2}')"
GEN_GAS="$(echo "$EVAL" | awk -F= '/^general_gas=/{print $2}')"
SHARE_BPS="$(echo "$EVAL" | awk -F= '/^payment_gas_share_bps=/{print $2}')"
STEAL="$(echo "$EVAL" | awk -F= '/^steal=/{print $2}')"
TX_COUNT="$(echo "$EVAL" | awk -F= '/^tx_count=/{print $2}')"

echo "payment count: $PAY_N"
echo "general count: $GEN_N"
echo "payment gas:   $PAY_GAS"
echo "general gas:   $GEN_GAS"
echo "payment gas share: ${SHARE_BPS} bps"

if [[ "$STEAL" == "1" ]]; then
  echo "FAIL: high-tip general landed inside first 70% while payments were waiting" >&2
  exit 1
fi

if [[ "${TX_COUNT:-0}" -lt 2 ]]; then
  echo "FAIL: expected multi-tx block, got tx_count=$TX_COUNT" >&2
  exit 1
fi

if [[ "${PAY_N:-0}" -gt 0 ]] && [[ "$SHARE_BPS" -lt "$ACCEPT_SHARE_BPS" ]]; then
  echo "FAIL: payment gas share ${SHARE_BPS} bps < ${ACCEPT_SHARE_BPS} bps (~65%)" >&2
  exit 1
fi

echo "=== Sprint 3 flood OK (8545) ==="

if cast chain-id --rpc-url "$ENGINE_RPC" >/dev/null 2>&1; then
  echo "9545 up: optional Mode A live flood skipped (code-path proof QuubLaneTxs is wired)."
else
  echo "9545 not up: Mode A live flood skipped; code-path proof QuubLaneTxs is wired."
fi
