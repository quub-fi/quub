#!/usr/bin/env bash
# Sprint 4: operator-api against quub-node --dev (eth_*).
# Prints: health JSON, payment tx hash, memoHash, freeze tx, second payment 422 reason 1.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

RPC="${QUUB_RPC:-http://127.0.0.1:8545}"
API="${QUUB_OPERATOR_API:-http://127.0.0.1:8080}"
export QUUB_RPC="$RPC"
export QUUB_OPERATOR_TOKEN="${QUUB_OPERATOR_TOKEN:-dev-operator-token}"
# Public anvil account 0 — --dev only. Not production.
export QUUB_OPERATOR_KEY="${QUUB_OPERATOR_KEY:-0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80}"
DEV_ADDR="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
AUTH="Authorization: Bearer ${QUUB_OPERATOR_TOKEN}"

TO="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
AMOUNT="1000000"
# Fresh ids each run so a warm node does not collide.
SUFFIX="$(openssl rand -hex 16 2>/dev/null || echo "$(date +%s)")"
E2E="0x$(printf '%064s' "${SUFFIX}aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" | tr ' ' 'a' | head -c 64)"
E2E2="0x$(printf '%064s' "${SUFFIX}bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" | tr ' ' 'b' | head -c 64)"
UETR="0x22222222222222222222222222222222"
INSTR="0x3333333333333333333333333333333333333333333333333333333333333333"
TR_HASH="0x0000000000000000000000000000000000000000000000000000000000000000"
PACK_HASH="0x4444444444444444444444444444444444444444444444444444444444444444"

NODE_STARTED=0
API_STARTED=0
NODE_PID=""
API_PID=""

cleanup() {
  if [[ "$API_STARTED" -eq 1 && -n "${API_PID}" ]] && kill -0 "$API_PID" 2>/dev/null; then
    kill "$API_PID" 2>/dev/null || true
    wait "$API_PID" 2>/dev/null || true
  fi
  if [[ "$NODE_STARTED" -eq 1 && -n "${NODE_PID}" ]] && kill -0 "$NODE_PID" 2>/dev/null; then
    kill "$NODE_PID" 2>/dev/null || true
    wait "$NODE_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

rpc_ready() {
  cast chain-id --rpc-url "$RPC" >/dev/null 2>&1
}

if ! rpc_ready; then
  echo "starting quub-node --dev on $RPC"
  cargo build -p quub-node 2>/dev/null || cargo build -p quub-node
  cargo run -p quub-node >/tmp/quub-gateway-node.log 2>&1 &
  NODE_PID=$!
  NODE_STARTED=1
  ready=0
  for _ in $(seq 1 90); do
    if rpc_ready; then
      ready=1
      break
    fi
    sleep 2
  done
  if [[ "$ready" -ne 1 ]]; then
    echo "RPC did not become ready on $RPC" >&2
    exit 1
  fi
fi

CHAIN_ID="$(cast chain-id --rpc-url "$RPC")"
echo "chain-id: $CHAIN_ID"
if [[ "$CHAIN_ID" != "8091" ]]; then
  echo "expected chain id 8091" >&2
  exit 1
fi

cargo build -p quub-operator-api
cargo run -p quub-operator-api >/tmp/quub-operator-api.log 2>&1 &
API_PID=$!
API_STARTED=1

health_ok=0
for _ in $(seq 1 60); do
  HEALTH="$(curl -sS "$API/health" || true)"
  if echo "$HEALTH" | grep -q '"ok":true' && echo "$HEALTH" | grep -q '8091'; then
    health_ok=1
    break
  fi
  sleep 1
done
if [[ "$health_ok" -ne 1 ]]; then
  echo "operator-api /health not ok" >&2
  echo "$HEALTH" >&2
  cat /tmp/quub-operator-api.log >&2 || true
  exit 1
fi
echo "=== health ==="
echo "$HEALTH"

pay_body() {
  local e2e="$1"
  cat <<EOF
{
  "to": "$TO",
  "amount": "$AMOUNT",
  "endToEndId": "$e2e",
  "uetr": "$UETR",
  "instrId": "$INSTR",
  "ccy": "USD",
  "msgType": 0,
  "trHash": "$TR_HASH",
  "packHash": "$PACK_HASH"
}
EOF
}

echo "=== POST /v1/payments ==="
PAY_RESP="$(curl -sS -w "\n%{http_code}" -X POST "$API/v1/payments" \
  -H "$AUTH" -H "Content-Type: application/json" \
  -d "$(pay_body "$E2E")")"
PAY_BODY="$(echo "$PAY_RESP" | sed '$d')"
PAY_CODE="$(echo "$PAY_RESP" | tail -n1)"
echo "http=$PAY_CODE body=$PAY_BODY"
if [[ "$PAY_CODE" != "202" ]]; then
  echo "expected 202 submitted" >&2
  exit 1
fi
TX_HASH="$(echo "$PAY_BODY" | python3 -c 'import sys,json; print(json.load(sys.stdin)["txHash"])')"
echo "payment tx hash: $TX_HASH"

echo "=== GET /v1/payments until mined ==="
MEMO=""
STATUS=""
for _ in $(seq 1 60); do
  GET_RESP="$(curl -sS -H "$AUTH" "$API/v1/payments/$E2E")"
  STATUS="$(echo "$GET_RESP" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("status",""))' 2>/dev/null || true)"
  MEMO="$(echo "$GET_RESP" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("memoHash") or "")' 2>/dev/null || true)"
  if [[ "$STATUS" == "mined" && -n "$MEMO" && "$MEMO" != "None" ]]; then
    break
  fi
  if [[ "$STATUS" == "failed" ]]; then
    echo "payment failed: $GET_RESP" >&2
    exit 1
  fi
  sleep 1
done
if [[ "$STATUS" != "mined" || -z "$MEMO" ]]; then
  echo "payment not mined with memoHash: status=$STATUS body=$GET_RESP" >&2
  exit 1
fi
echo "status: mined"
echo "memoHash: $MEMO"

echo "=== POST /v1/freeze (operator) ==="
FZ_RESP="$(curl -sS -X POST "$API/v1/freeze" \
  -H "$AUTH" -H "Content-Type: application/json" \
  -d "{\"address\":\"$DEV_ADDR\"}")"
echo "$FZ_RESP"
FZ_TX="$(echo "$FZ_RESP" | python3 -c 'import sys,json; print(json.load(sys.stdin)["txHash"])')"
echo "freeze tx: $FZ_TX"
# Wait for freeze to mine
for _ in $(seq 1 30); do
  ST="$(cast receipt "$FZ_TX" --rpc-url "$RPC" 2>/dev/null | awk '/^status/{print $2; exit}' || true)"
  if [[ "$ST" == "1" || "$ST" == "0x1" ]]; then
    break
  fi
  sleep 1
done

echo "=== POST /v1/payments (expect 422 reason 1) ==="
PAY2_RESP="$(curl -sS -w "\n%{http_code}" -X POST "$API/v1/payments" \
  -H "$AUTH" -H "Content-Type: application/json" \
  -d "$(pay_body "$E2E2")")"
PAY2_BODY="$(echo "$PAY2_RESP" | sed '$d')"
PAY2_CODE="$(echo "$PAY2_RESP" | tail -n1)"
echo "http=$PAY2_CODE body=$PAY2_BODY"
if [[ "$PAY2_CODE" != "422" ]]; then
  echo "expected 422 policy_rejected" >&2
  exit 1
fi
REASON="$(echo "$PAY2_BODY" | python3 -c 'import sys,json; print(json.load(sys.stdin)["reason"])')"
if [[ "$REASON" != "1" ]]; then
  echo "expected reason 1, got $REASON" >&2
  exit 1
fi
echo "second payment rejected: 422 / reason 1"

echo "=== POST /v1/unfreeze (freeze is not admin-lock) ==="
UF_RESP="$(curl -sS -w "\n%{http_code}" -X POST "$API/v1/unfreeze" \
  -H "$AUTH" -H "Content-Type: application/json" \
  -d "{\"address\":\"$DEV_ADDR\"}")"
UF_BODY="$(echo "$UF_RESP" | sed '$d')"
UF_CODE="$(echo "$UF_RESP" | tail -n1)"
echo "http=$UF_CODE body=$UF_BODY"
if [[ "$UF_CODE" != "200" ]]; then
  echo "unfreeze must succeed after self-freeze (F211 owner path)" >&2
  exit 1
fi

echo "=== Sprint 4 gateway-dev OK ==="
echo "health: $HEALTH"
echo "payment tx hash: $TX_HASH"
echo "memoHash: $MEMO"
echo "freeze tx: $FZ_TX"
echo "second payment: 422 reason 1"
