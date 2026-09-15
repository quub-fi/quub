#!/usr/bin/env bash
# Sprint 6: Quub memo on 8091, then CCTP V2 Sepolia→Base Sepolia (live) or mock.
# Quub tx is identity only — never the USDC burn. See specs/rail-cctp.md / ADR-020.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export PATH="${HOME}/.foundry/bin:${HOME}/.cargo/bin:${PATH:-}"
export CARGO_HOME="${CARGO_HOME:-/tmp/quub-cargo-home-adr016}"
if [[ -d "${HOME}/.rustup/toolchains/1.96.0-aarch64-apple-darwin/bin" ]]; then
  export PATH="${HOME}/.rustup/toolchains/1.96.0-aarch64-apple-darwin/bin:${PATH}"
fi

RPC="${QUUB_RPC:-http://127.0.0.1:8545}"
DEV_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
DEV_ADDR="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
TO="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"

F210="0x000000000000000000000000000000000000F210"
F202="0x000000000000000000000000000000000000F202"

# Circle CCTP V2 pins (specs/rail-cctp.md)
TOKEN_MESSENGER_V2="0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA"
MESSAGE_TRANSMITTER_V2="0xE737e5cEBEEBa77EFE34D4aa090756590b1CE275"
USDC_ETH_SEPOLIA="0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"
ETH_SEPOLIA_DOMAIN=0
BASE_SEPOLIA_DOMAIN=6

ART_DIR="$ROOT/artifacts/rail-base"
mkdir -p "$ART_DIR"

# Stable e2e for idempotent retries; override with RAIL_E2E.
E2E="${RAIL_E2E:-0x6666666666666666666666666666666666666666666666666666666666666666}"
UETR="0x22222222222222222222222222222222"
INSTR="0x3333333333333333333333333333333333333333333333333333333333333333"
CCY="0x555344"
MSG_TYPE="0"
TR_HASH="0x0000000000000000000000000000000000000000000000000000000000000000"
PACK_HASH="0x4444444444444444444444444444444444444444444444444444444444444444"
AMOUNT="1000000"
# ≤1 USDC on Sepolia when live (6 decimals)
CCTP_AMOUNT="${CCTP_AMOUNT:-1000000}"

REC_FILE="$ART_DIR/${E2E#0x}.json"

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
  cargo build -p quub-node 2>&1 | tail -3
  cargo run -p quub-node --quiet >/tmp/quub-rail-base-node.log 2>&1 &
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
[[ "$CHAIN_ID" == "8091" ]] || { echo "expected Quub 8091, got $CHAIN_ID" >&2; exit 1; }

LIVE=0
if [[ -n "${SEPOLIA_RPC:-}" && -n "${BASE_SEPOLIA_RPC:-}" && -n "${CIRCLE_API:-}" && -n "${CCTP_BURNER_KEY:-}" ]]; then
  LIVE=1
  MODE="live"
else
  MODE="mock"
fi
echo "mode=$MODE"

# --- (1) Quub transferWithMemo (identity) — or reuse stored record ---
QUUB_TX=""
MEMO_HASH=""
if [[ -f "$REC_FILE" ]]; then
  QUUB_TX="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("quub_tx",""))' "$REC_FILE")"
  MEMO_HASH="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("memo_hash",""))' "$REC_FILE")"
  PREV_BURN="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("burn_tx") or "")' "$REC_FILE")"
  PREV_STATUS="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("status",""))' "$REC_FILE")"
  if [[ -n "$QUUB_TX" && -n "$MEMO_HASH" ]]; then
    echo "reuse Quub record for endToEndId (no second memo)"
    echo "quub_tx: $QUUB_TX"
    echo "memoHash: $MEMO_HASH"
  fi
fi

if [[ -z "$QUUB_TX" || -z "$MEMO_HASH" ]]; then
  echo "=== Quub transferWithMemo on 8091 ==="
  TX_OUT="$(cast send "$F210" \
    "transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)" \
    "$TO" "$AMOUNT" "$E2E" "$UETR" "$INSTR" "$CCY" "$MSG_TYPE" "$TR_HASH" "$PACK_HASH" \
    --rpc-url "$RPC" --private-key "$DEV_KEY" \
    --legacy --gas-price 1000000000 --gas-limit 800000 2>&1)"
  QUUB_TX="$(echo "$TX_OUT" | awk '/^transactionHash/{print $2; exit}')"
  [[ -n "$QUUB_TX" ]] || { echo "$TX_OUT" >&2; echo "no Quub tx hash" >&2; exit 1; }
  RECEIPT="$(cast receipt "$QUUB_TX" --rpc-url "$RPC")"
  echo "$RECEIPT" | grep -q "success" || { echo "Quub memo failed" >&2; exit 1; }
  MEMO_HASH="$(cast call "$F202" \
    "validateAndCommit(bytes32,bytes16,bytes32,bytes3,uint8)(bytes32)" \
    "$E2E" "$UETR" "$INSTR" "$CCY" "$MSG_TYPE" \
    --from "$DEV_ADDR" --rpc-url "$RPC")"
  echo "quub_tx: $QUUB_TX"
  echo "memoHash: $MEMO_HASH"
  python3 - "$REC_FILE" "$E2E" "$QUUB_TX" "$MEMO_HASH" <<'PY'
import json, sys
path, e2e, quub, memo = sys.argv[1:5]
rec = {
  "end_to_end_id": e2e,
  "quub_tx": quub,
  "memo_hash": memo,
  "burn_tx": None,
  "mint_tx": None,
  "status": "quub_only",
  "dest": "mock://base-sepolia",
}
json.dump(rec, open(path, "w"), indent=2)
print("wrote", path)
PY
  PREV_BURN=""
  PREV_STATUS="quub_only"
fi

# Normalize hex for compare
norm() { echo "$1" | tr '[:upper:]' '[:lower:]'; }

# --- (2)+(3) CCTP burn + attest + mint (live or mock); no second Quub memo ---
BURN_TX="${PREV_BURN:-}"
MINT_TX=""

if [[ "${PREV_STATUS:-}" == "minted" ]]; then
  MINT_TX="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("mint_tx") or "")' "$REC_FILE")"
  BURN_TX="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("burn_tx") or "")' "$REC_FILE")"
  echo "already minted — idempotent return"
elif [[ "$LIVE" -eq 1 ]]; then
  echo "=== CCTP live: burn Sepolia USDC via TokenMessengerV2 ==="
  if [[ -z "$BURN_TX" ]]; then
    # Approve + depositForBurn (amount, destDomain, mintRecipient, burnToken)
    cast send "$USDC_ETH_SEPOLIA" "approve(address,uint256)" \
      "$TOKEN_MESSENGER_V2" "$CCTP_AMOUNT" \
      --rpc-url "$SEPOLIA_RPC" --private-key "$CCTP_BURNER_KEY" >/dev/null
    MINT_RECIPIENT="$(cast wallet address --private-key "$CCTP_BURNER_KEY")"
    # bytes32 mintRecipient = left-padded address
    MINT_BYTES32="$(python3 -c "a='$MINT_RECIPIENT'.lower().replace('0x',''); print('0x'+a.rjust(64,'0'))")"
    BURN_OUT="$(cast send "$TOKEN_MESSENGER_V2" \
      "depositForBurn(uint256,uint32,bytes32,address)" \
      "$CCTP_AMOUNT" "$BASE_SEPOLIA_DOMAIN" "$MINT_BYTES32" "$USDC_ETH_SEPOLIA" \
      --rpc-url "$SEPOLIA_RPC" --private-key "$CCTP_BURNER_KEY" 2>&1)" || {
      echo "CCTP burn failed — keeping Quub memo; status=failed" >&2
      python3 - "$REC_FILE" <<'PY'
import json, sys
p = sys.argv[1]
r = json.load(open(p))
r["status"] = "failed"
json.dump(r, open(p, "w"), indent=2)
PY
      exit 1
    }
    BURN_TX="$(echo "$BURN_OUT" | awk '/^transactionHash/{print $2; exit}')"
    [[ -n "$BURN_TX" ]] || { echo "$BURN_OUT" >&2; exit 1; }
    python3 - "$REC_FILE" "$BURN_TX" <<'PY'
import json, sys
p, burn = sys.argv[1:3]
r = json.load(open(p))
r["burn_tx"] = burn
r["status"] = "burned"
r["dest"] = "base-sepolia"
json.dump(r, open(p, "w"), indent=2)
PY
  else
    echo "reuse burn_tx (no double-burn): $BURN_TX"
  fi

  echo "=== attest via CIRCLE_API ==="
  # CIRCLE_API = base URL e.g. https://iris-api-sandbox.circle.com
  ATT_URL="${CIRCLE_API%/}/v2/messages/${ETH_SEPOLIA_DOMAIN}?transactionHash=${BURN_TX}"
  MESSAGE=""
  ATTESTATION=""
  for _ in $(seq 1 60); do
    RESP="$(curl -sS "$ATT_URL" || true)"
    MESSAGE="$(echo "$RESP" | python3 -c 'import sys,json
try:
 d=json.load(sys.stdin); m=d.get("messages") or d.get("data") or []
 if m: print(m[0].get("message") or m[0].get("messageBytes") or "")
except Exception: print("")
' 2>/dev/null || true)"
    ATTESTATION="$(echo "$RESP" | python3 -c 'import sys,json
try:
 d=json.load(sys.stdin); m=d.get("messages") or d.get("data") or []
 if m: print(m[0].get("attestation") or "")
except Exception: print("")
' 2>/dev/null || true)"
    if [[ -n "$MESSAGE" && -n "$ATTESTATION" && "$ATTESTATION" != "PENDING" ]]; then
      break
    fi
    sleep 5
  done
  if [[ -z "$MESSAGE" || -z "$ATTESTATION" || "$ATTESTATION" == "PENDING" ]]; then
    echo "attestation not ready — status=failed (Quub memo kept)" >&2
    python3 - "$REC_FILE" <<'PY'
import json, sys
p = sys.argv[1]
r = json.load(open(p))
r["status"] = "failed"
json.dump(r, open(p, "w"), indent=2)
PY
    exit 1
  fi

  echo "=== mint on Base Sepolia MessageTransmitterV2 ==="
  MINT_OUT="$(cast send "$MESSAGE_TRANSMITTER_V2" \
    "receiveMessage(bytes,bytes)" "$MESSAGE" "$ATTESTATION" \
    --rpc-url "$BASE_SEPOLIA_RPC" --private-key "$CCTP_BURNER_KEY" 2>&1)" || {
    echo "mint failed — status=failed (Quub memo kept)" >&2
    python3 - "$REC_FILE" <<'PY'
import json, sys
p = sys.argv[1]
r = json.load(open(p))
r["status"] = "failed"
json.dump(r, open(p, "w"), indent=2)
PY
    exit 1
  }
  MINT_TX="$(echo "$MINT_OUT" | awk '/^transactionHash/{print $2; exit}')"
  [[ -n "$MINT_TX" ]] || { echo "$MINT_OUT" >&2; exit 1; }
else
  echo "=== CCTP mock (no SEPOLIA_RPC / BASE_SEPOLIA_RPC / CIRCLE_API / CCTP_BURNER_KEY) ==="
  if [[ -z "$BURN_TX" ]]; then
    # Synthetic burn ≠ Quub: prefix 0xccb1… (matches quub-gateway mock)
    NONCE="$(date +%s)"
    BURN_TX="$(python3 -c "
n=int('$NONCE')
b=bytearray(32); b[0]=0xcc; b[1]=0xb1
b[24:32]=n.to_bytes(8,'big')
print('0x'+b.hex())
")"
  fi
  NONCE2="$(date +%s)"
  MINT_TX="$(python3 -c "
n=int('$NONCE2')
b=bytearray(32); b[0]=0xcc; b[1]=0xb2
b[2:10]=n.to_bytes(8,'big')
print('0x'+b.hex())
")"
fi

# Hard fail: burn must never equal Quub identity tx
if [[ "$(norm "$BURN_TX")" == "$(norm "$QUUB_TX")" ]]; then
  echo "FAIL: burn_id == quub_tx — Quub 8091 is not a CCTP domain" >&2
  exit 1
fi

python3 - "$REC_FILE" "$BURN_TX" "$MINT_TX" "$MODE" <<'PY'
import json, sys
p, burn, mint, mode = sys.argv[1:5]
r = json.load(open(p))
r["burn_tx"] = burn
r["mint_tx"] = mint
r["status"] = "minted"
r["mode"] = mode
r["dest"] = "base-sepolia" if mode == "live" else "mock://base-sepolia"
json.dump(r, open(p, "w"), indent=2)
print("wrote", p)
PY

echo "=== Sprint 6 rail-base OK ==="
echo "mode=$MODE"
echo "quub_tx: $QUUB_TX"
echo "memoHash: $MEMO_HASH"
echo "burn_id: $BURN_TX"
echo "mint_id: $MINT_TX"
