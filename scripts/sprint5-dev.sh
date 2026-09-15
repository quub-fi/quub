#!/usr/bin/env bash
# Sprint 5: --dev fee on memo + two-key unfreeze (OwnerA=anvil0, OwnerB=anvil1).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export PATH="${HOME}/.foundry/bin:${HOME}/.cargo/bin:${PATH:-}"
export CARGO_HOME="${CARGO_HOME:-/tmp/quub-cargo-home-adr016}"
export PATH="${HOME}/.rustup/toolchains/1.96.0-aarch64-apple-darwin/bin:${PATH}"

RPC="http://127.0.0.1:8545"
DEV_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
DEV_ADDR="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
# anvil1 — OwnerB on --dev genesis (ADR-019)
OWNER_B_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
OWNER_B_ADDR="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"

F210="0x000000000000000000000000000000000000F210"
F211="0x000000000000000000000000000000000000F211"
F213="0x000000000000000000000000000000000000F213"
TO="$OWNER_B_ADDR"

E2E="0x5555555555555555555555555555555555555555555555555555555555555555"
UETR="0x22222222222222222222222222222222"
INSTR="0x3333333333333333333333333333333333333333333333333333333333333333"
CCY="0x555344"
MSG_TYPE="0"
TR_HASH="0x0000000000000000000000000000000000000000000000000000000000000000"
PACK_HASH="0x0000000000000000000000000000000000000000000000000000000000000000"
AMOUNT="1000000"

FEE_GAS_LIMIT=21000
FEE_GAS_PRICE=1

cleanup() {
  if [[ -n "${NODE_PID:-}" ]] && kill -0 "$NODE_PID" 2>/dev/null; then
    kill "$NODE_PID" 2>/dev/null || true
    wait "$NODE_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

STARTED_NODE=0
if ! cast chain-id --rpc-url "$RPC" >/dev/null 2>&1; then
  if command -v lsof >/dev/null 2>&1; then
    lsof -tiTCP:8545 -sTCP:LISTEN 2>/dev/null | xargs kill 2>/dev/null || true
  fi
  pkill -f 'target/.*/quub-node' 2>/dev/null || true
  sleep 1
  echo "=== starting quub-node --dev ==="
  cargo build -p quub-node 2>&1 | tail -5
  cargo run -p quub-node --quiet &
  NODE_PID=$!
  STARTED_NODE=1
  ready=0
  for _ in $(seq 1 120); do
    if cast chain-id --rpc-url "$RPC" >/dev/null 2>&1; then
      ready=1
      break
    fi
    sleep 1
  done
  [[ "$ready" -eq 1 ]] || { echo "RPC not ready" >&2; exit 1; }
fi

CHAIN_ID="$(cast chain-id --rpc-url "$RPC")"
[[ "$CHAIN_ID" == "8091" ]] || { echo "expected 8091" >&2; exit 1; }

OWNER_A="$(cast call "$F211" "ownerA()(address)" --rpc-url "$RPC" | tr '[:upper:]' '[:lower:]')"
OWNER_B="$(cast call "$F211" "ownerB()(address)" --rpc-url "$RPC" | tr '[:upper:]' '[:lower:]')"
FEE_RECIPIENT="$(cast call "$F213" "feeRecipient()(address)" --rpc-url "$RPC")"
echo "ownerA: $OWNER_A"
echo "ownerB: $OWNER_B"
echo "feeRecipient: $FEE_RECIPIENT"

OA_L="$(echo "$DEV_ADDR" | tr '[:upper:]' '[:lower:]')"
OB_L="$(echo "$OWNER_B_ADDR" | tr '[:upper:]' '[:lower:]')"
if [[ "$OWNER_A" != "$OA_L" ]] || [[ "$OWNER_B" != "$OB_L" ]]; then
  echo "FAIL: expected OwnerA=anvil0 OwnerB=anvil1 on --dev" >&2
  exit 1
fi
if [[ "$OWNER_A" == "$OWNER_B" ]]; then
  echo "FAIL: OwnerB == OwnerA (dual-control bypass)" >&2
  exit 1
fi
echo "OwnerB != OwnerA live? yes"

QUOTE="$(cast call "$F213" "quote(address,uint256,uint256)(uint256)" \
  "$F210" "$FEE_GAS_LIMIT" "$FEE_GAS_PRICE" --rpc-url "$RPC")"
echo "quote: $QUOTE"

bal() { cast call "$F210" "balanceOf(address)(uint256)" "$1" --rpc-url "$RPC"; }

SENDER_BEFORE="$(bal "$DEV_ADDR")"
TO_BEFORE="$(bal "$TO")"
FEE_BEFORE="$(bal "$FEE_RECIPIENT")"
echo "=== balances before ==="
echo "sender:       $SENDER_BEFORE"
echo "recipient:    $TO_BEFORE"
echo "feeRecipient: $FEE_BEFORE"

TX_OUT="$(cast send "$F210" \
  "transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)" \
  "$TO" "$AMOUNT" "$E2E" "$UETR" "$INSTR" "$CCY" "$MSG_TYPE" "$TR_HASH" "$PACK_HASH" \
  --rpc-url "$RPC" --private-key "$DEV_KEY" \
  --legacy --gas-price 1000000000 --gas-limit 800000 2>&1)"
TX_HASH="$(echo "$TX_OUT" | awk '/^transactionHash/{print $2; exit}')"
[[ -n "$TX_HASH" ]] || { echo "$TX_OUT" >&2; echo "no tx hash" >&2; exit 1; }
echo "memo tx: $TX_HASH"
RECEIPT="$(cast receipt "$TX_HASH" --rpc-url "$RPC")"
echo "$RECEIPT" | grep -q "success" || { echo "memo failed" >&2; exit 1; }

SENDER_AFTER="$(bal "$DEV_ADDR")"
TO_AFTER="$(bal "$TO")"
FEE_AFTER="$(bal "$FEE_RECIPIENT")"
echo "=== balances after ==="
echo "sender:       $SENDER_AFTER"
echo "recipient:    $TO_AFTER"
echo "feeRecipient: $FEE_AFTER"

# Parse uint256 (decimal or hex)
to_dec() {
  python3 -c "import sys; s=sys.argv[1].strip().split()[0]; print(int(s,0))" "$1"
}
FEE_DELTA=$(( $(to_dec "$FEE_AFTER") - $(to_dec "$FEE_BEFORE") ))
QUOTE_DEC="$(to_dec "$QUOTE")"
echo "fee delta: $FEE_DELTA"
echo "quote:     $QUOTE_DEC"
DIFF=$(( FEE_DELTA - QUOTE_DEC ))
if [[ ${DIFF#-} -gt 1 ]]; then
  echo "FAIL: fee delta $FEE_DELTA != quote $QUOTE_DEC (±1)" >&2
  exit 1
fi
SENDER_DELTA=$(( $(to_dec "$SENDER_BEFORE") - $(to_dec "$SENDER_AFTER") ))
AMOUNT_DEC="$(to_dec "$AMOUNT")"
EXPECTED_SENDER=$(( AMOUNT_DEC + QUOTE_DEC ))
SDIFF=$(( SENDER_DELTA - EXPECTED_SENDER ))
if [[ ${SDIFF#-} -gt 1 ]]; then
  echo "FAIL: sender lost $SENDER_DELTA, expected principal+fee $EXPECTED_SENDER" >&2
  exit 1
fi

echo "=== freeze from OwnerA ==="
cast send "$F211" "freeze(address)" "$DEV_ADDR" \
  --rpc-url "$RPC" --private-key "$DEV_KEY" --legacy --gas-price 1000000000 >/dev/null

echo "=== unfreeze from OwnerA alone (must fail) ==="
# Instant unfreeze removed — propose then same-key confirm must fail.
cast send "$F211" "proposeUnfreeze(address)" "$DEV_ADDR" \
  --rpc-url "$RPC" --private-key "$DEV_KEY" --legacy --gas-price 1000000000 >/dev/null
set +e
ALONE_OUT="$(cast send "$F211" "confirmUnfreeze(address)" "$DEV_ADDR" \
  --rpc-url "$RPC" --private-key "$DEV_KEY" --legacy --gas-price 1000000000 2>&1)"
ALONE_EC=$?
set -e
# Even if cast exits 0, mined tx may revert (status 0x0)
ALONE_HASH="$(echo "$ALONE_OUT" | awk '/^transactionHash/{print $2; exit}')"
if [[ -n "$ALONE_HASH" ]]; then
  ST="$(cast receipt "$ALONE_HASH" --rpc-url "$RPC" --json 2>/dev/null | python3 -c 'import json,sys; w=json.load(sys.stdin); d=w.get("data",w); print(d.get("status","") )' 2>/dev/null || true)"
  if [[ "$ST" == "0x1" || "$ST" == "1" ]]; then
    echo "FAIL: anvil0-alone confirmUnfreeze succeeded" >&2
    exit 1
  fi
fi
# Also try legacy unfreeze selector — must not exist / must fail
set +e
LEGACY="$(cast send "$F211" "unfreeze(address)" "$DEV_ADDR" \
  --rpc-url "$RPC" --private-key "$DEV_KEY" --legacy --gas-price 1000000000 2>&1)"
set -e
if echo "$LEGACY" | grep -qi "transactionHash"; then
  LH="$(echo "$LEGACY" | awk '/^transactionHash/{print $2; exit}')"
  LST="$(cast receipt "$LH" --rpc-url "$RPC" --json 2>/dev/null | python3 -c 'import json,sys; w=json.load(sys.stdin); d=w.get("data",w); print(d.get("status","") )' 2>/dev/null || echo fail)"
  if [[ "$LST" == "0x1" || "$LST" == "1" ]]; then
    echo "FAIL: public instant unfreeze still works" >&2
    exit 1
  fi
fi
echo "anvil0-alone unfreeze: rejected (ok)"

echo "=== confirmUnfreeze from OwnerB (anvil1) ==="
cast send "$F211" "confirmUnfreeze(address)" "$DEV_ADDR" \
  --rpc-url "$RPC" --private-key "$OWNER_B_KEY" --legacy --gas-price 1000000000 >/dev/null
FROZEN="$(cast call "$F211" "frozen(address)(bool)" "$DEV_ADDR" --rpc-url "$RPC")"
echo "frozen after B confirm: $FROZEN"
[[ "$FROZEN" == "false" ]] || { echo "FAIL: still frozen" >&2; exit 1; }

echo "=== Sprint 5 --dev OK ==="
echo "fee delta: $FEE_DELTA"
echo "quote: $QUOTE_DEC"
echo "Foundry two-owner unfreeze test: test_unfreeze_proposeA_confirmB"
echo "OwnerB != OwnerA live? yes"
[[ "$STARTED_NODE" -eq 1 ]] || true
