#!/usr/bin/env bash
# Sprint 1.5: start quub-node, assert F210–F213, mine one transferWithMemo, print receipt.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

RPC="http://127.0.0.1:8545"
# Public anvil account 0 — --dev only. Not production.
DEV_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
DEV_ADDR="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

F201="0x000000000000000000000000000000000000F201"
F202="0x000000000000000000000000000000000000F202"
F203="0x000000000000000000000000000000000000F203"
F210="0x000000000000000000000000000000000000F210"
F211="0x000000000000000000000000000000000000F211"
F212="0x000000000000000000000000000000000000F212"
F213="0x000000000000000000000000000000000000F213"

# Sprint 0 usd_pacs008 identity set (fixture hash uses origin 0x…00AA; mined uses DEV_ADDR).
E2E="0x1111111111111111111111111111111111111111111111111111111111111111"
UETR="0x22222222222222222222222222222222"
INSTR="0x3333333333333333333333333333333333333333333333333333333333333333"
CCY="0x555344" # USD
MSG_TYPE="0"
TR_HASH="0x0000000000000000000000000000000000000000000000000000000000000000"
PACK_HASH="0x4444444444444444444444444444444444444444444444444444444444444444"
TO="0x70997970C51812dc3A010C7d01b50e0d17dc79C8" # anvil account 1
AMOUNT="1000000" # 1e6 units

cleanup() {
  if [[ -n "${NODE_PID:-}" ]] && kill -0 "$NODE_PID" 2>/dev/null; then
    kill "$NODE_PID" 2>/dev/null || true
    wait "$NODE_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

if command -v lsof >/dev/null 2>&1; then
  lsof -tiTCP:8545 -sTCP:LISTEN 2>/dev/null | xargs kill 2>/dev/null || true
  lsof -tiTCP:8551 -sTCP:LISTEN 2>/dev/null | xargs kill 2>/dev/null || true
fi
pkill -f 'target/debug/quub-node' 2>/dev/null || true
sleep 1

cargo run -p quub-node &
NODE_PID=$!

ready=0
for _ in $(seq 1 90); do
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

echo "=== datadir ==="
echo "ephemeral NodeConfig::test() datadir (see quub-node stderr); receipts do not survive restart"

echo "=== contract addresses ==="
echo "F201 (policy precompile):  $F201"
echo "F202 (iso memo precompile): $F202"
echo "F203 (paymaster precompile): $F203"
echo "F210 PaymentToken:         $F210"
echo "F211 PolicyAdmin:          $F211"
echo "F212 EvidenceAnchor:       $F212"
echo "F213 PaymasterEntry:       $F213"

for addr in "$F210" "$F211" "$F212" "$F213"; do
  code="$(cast code "$addr" --rpc-url "$RPC")"
  if [[ "$code" == "0x" || -z "$code" ]]; then
    echo "expected non-empty code at $addr" >&2
    exit 1
  fi
  codesize="$(cast codesize "$addr" --rpc-url "$RPC" 2>/dev/null || echo "?")"
  echo "cast code $addr: ${#code} hex chars; codesize=$codesize"
done

F213_SIZE="$(cast codesize "$F213" --rpc-url "$RPC")"
if [[ "$F213_SIZE" == "0" || -z "$F213_SIZE" ]]; then
  echo "F213 expected non-zero codesize (genesis etch PaymasterEntry)" >&2
  exit 1
fi
echo "F213 codesize: $F213_SIZE"

BAL="$(cast call "$F210" "balanceOf(address)(uint256)" "$DEV_ADDR" --rpc-url "$RPC")"
echo "dev balanceOf: $BAL"

TX_OUT="$(cast send "$F210" \
  "transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)" \
  "$TO" "$AMOUNT" "$E2E" "$UETR" "$INSTR" "$CCY" "$MSG_TYPE" "$TR_HASH" "$PACK_HASH" \
  --rpc-url "$RPC" \
  --private-key "$DEV_KEY" \
  --legacy --gas-price 1000000000 --gas-limit 500000 2>&1)"
echo "$TX_OUT"
# Prefer the receipt field line (not the nested logs JSON).
TX_HASH="$(echo "$TX_OUT" | awk '/^transactionHash/{print $2; exit}')"
if [[ -z "$TX_HASH" ]]; then
  echo "failed to parse tx hash" >&2
  exit 1
fi
echo "tx hash: $TX_HASH"

RECEIPT="$(cast receipt "$TX_HASH" --rpc-url "$RPC")"
echo "$RECEIPT"
if ! echo "$RECEIPT" | grep -q "success"; then
  echo "transferWithMemo failed" >&2
  exit 1
fi

MEMO_CALL="$(cast call "$F202" \
  "validateAndCommit(bytes32,bytes16,bytes32,bytes3,uint8)(bytes32)" \
  "$E2E" "$UETR" "$INSTR" "$CCY" "$MSG_TYPE" \
  --from "$DEV_ADDR" \
  --rpc-url "$RPC")"
echo "F202 memoHash (origin=$DEV_ADDR): $MEMO_CALL"
echo "note: usd_pacs008_stable_hash uses origin 0x…00AA and will NOT equal this mined hash"

echo "=== Sprint 1.5 pay OK ==="
echo "tx hash: $TX_HASH"
