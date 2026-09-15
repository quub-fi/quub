# ISO memo canonicalization (F202)

Quub Memo / Quub ISO memos are an **identity set + hash**. No names, IBANs, XML, or PII go on-chain. Full ISO 20022 documents stay in Quub Evidence.

## Off-chain EndToEndId

- Encoding: UTF-8
- Max length: 35 characters (ISO 20022 EndToEndId bound)
- On-chain / precompile input: `bytes32 endToEndId = keccak256(utf8_bytes)`
- Empty string or missing EndToEndId is invalid → revert (do not hash then check)

## Message types

| `msgType` | Message   |
| --------- | --------- |
| 0         | pacs.008  |
| 1         | pain.001  |
| 2         | pacs.009  |
| 3         | camt.054  |

Rules:

- `endToEndId != 0`
- if `msgType` is 0 or 2 (`pacs.008` / `pacs.009`) then `uetr != 0`
- year-1 `ccy` ∈ `{USD, CAD, AED, SAR}` as ASCII `bytes3`

## Hash

```
memoHash = keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, tx.origin))
```

- **No `block.timestamp` in the hash.** Same inputs always produce the same hash.
- `tx.origin` (or the off-chain submitting party address in Quub Gateway / Quub ISO) is part of the preimage so the memo is bound to the initiator.
- Gas target for the precompile path: 2_500.

## Evidence

Anchor `(packHash, memoHash)` via Quub Evidence / F212. The pack is off-chain; the chain only stores hashes.
