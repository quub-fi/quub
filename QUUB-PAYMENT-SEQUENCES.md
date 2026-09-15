# Quub — Payment use cases: interactions and sequences
**Chain:** 8091. **Token on Quub:** F210 (QPT dummy on `--dev`). **Dollars off-Quub:** USDC via Circle CCTP.  
**Actors:** Originator, Operator API `:8080`, `quub-iso`, `quub-node`, F201–F213, OwnerA / OwnerB, `quub-gateway`, Circle CCTP, Base (or Sepolia).

Addresses:

| Addr | Role |
|---|---|
| F201 | Policy precompile (caller = F210 only) |
| F202 | ISO memo precompile (hash includes `tx.origin`) |
| F203 | Paymaster precompile (thin) |
| F210 | PaymentToken |
| F211 | PolicyAdmin |
| F212 | EvidenceAnchor |
| F213 | PaymasterEntry |

---

## UC1 — On-us identified payment (happy path)

Originator pays another address on Quub only. No CCTP. This is the Sprint 1.5 / 5 path.

```mermaid
sequenceDiagram
  autonumber
  actor Orig as Originator (anvil0)
  participant Node as quub-node :8545/:9545
  participant Pool as quub-pool lane
  participant F210 as PaymentToken
  participant F201 as Policy F201
  participant F211 as PolicyAdmin
  participant F213 as PaymasterEntry
  participant F202 as ISO memo F202
  participant F212 as EvidenceAnchor

  Orig->>Node: transferWithMemo(to, amt, e2e, uetr, instr, ccy, msg, tr, pack)
  Node->>Pool: classify to==F210 + exact selector → payment lane
  Pool-->>Node: include in ≤70% payment gas
  Node->>F210: CALL
  F210->>F201: check(from, to, amt, …)
  F201->>F211: SLOAD frozen / pause / threshold
  F211-->>F201: not frozen
  F201-->>F210: reason 0 allow
  F210->>F213: quote(F210, FEE_GAS, FEE_PRICE)
  F213-->>F210: 21000
  F210->>F213: takeFee(payer, F210, 21000)
  F213->>F210: paymasterDebit(payer, feeRecipient, 21000)
  Note over F210: _move fee only — no policy, no nested takeFee
  F210->>F210: _move principal payer → to
  F210->>F202: validateAndCommit(fields)
  F202-->>F210: memoHash = keccak256(abi.encode(e2e,uetr,instr,ccy,msg,tx.origin))
  F210->>F212: anchor(packHash, memoHash)
  F210-->>Orig: MemoAnchored + EvidenceAnchored + Transfer×2 (fee, principal)
```

**Fails closed if:** F201 ≠ 0, F213 unlisted token, F201 called by EOA, calldata length ≠ memo.

---

## UC2 — Same payment through the operator API

Bank/ops does not speak Solidity. Sprint 4.

```mermaid
sequenceDiagram
  autonumber
  actor Ops as Ops / core adapter
  participant API as operator-api :8080
  participant ISO as quub-iso
  participant Id as idempotency map
  participant Node as quub-node
  participant F210 as F210…F213

  Ops->>API: POST /v1/payments  Bearer token<br/>{to, amount, endToEndId, uetr, instrId, ccy, msgType, trHash, packHash}
  API->>API: 401 if bearer wrong
  API->>ISO: validate_and_commit (origin = operator signer)
  alt empty e2e / missing UETR / bad ccy
    ISO-->>API: err
    API-->>Ops: 400
  else ok
    API->>Id: lookup endToEndId
    alt already have txHash
      Id-->>API: existing
      API-->>Ops: 202 same txHash (no second send)
    else new
      API->>Node: eth_sendRawTransaction (operator key)
      Note over Node,F210: UC1 from here
      Node-->>API: txHash
      API->>Id: store after successful broadcast only
      API-->>Ops: 202 {id, txHash, status: submitted}
      Ops->>API: GET /v1/payments/{endToEndId}
      API->>Node: eth_getTransactionReceipt
      API-->>Ops: mined + memoHash from MemoAnchored
    end
  end
```

---

## UC3 — Policy reject (frozen sender)

Sanctions / stop-pay. Fee and principal must not move. Rail must not burn.

```mermaid
sequenceDiagram
  autonumber
  actor Owner as OwnerA
  actor Payer as Payer
  participant F211 as PolicyAdmin
  participant F210 as PaymentToken
  participant F201 as Policy F201
  participant F213 as PaymasterEntry
  participant CCTP as Circle CCTP

  Owner->>F211: freeze(payer)        %% immediate, either owner
  F211-->>Owner: status 1
  Payer->>F210: transferWithMemo(...)
  F210->>F201: check
  F201->>F211: SLOAD frozen[payer]
  F201-->>F210: reason 1 frozen_from
  F210-->>Payer: revert PolicyRejected(1)
  Note over F210,F213: no takeFee, no _move
  Note over CCTP: burn never called
```

API shape: `POST /v1/freeze` then `POST /v1/payments` → **422** `{error: policy_rejected, reason: 1, reasonName: frozen_from}`.

---

## UC4 — Unfreeze (maker-checker)

Sprint 5. Same key twice is not dual-control.

```mermaid
sequenceDiagram
  autonumber
  actor A as OwnerA anvil0
  actor B as OwnerB anvil1
  participant F211 as PolicyAdmin
  participant F210 as PaymentToken

  A->>F211: unfreeze(payer)
  F211-->>A: revert (no instant unfreeze)
  A->>F211: proposeUnfreeze(payer)
  F211-->>A: proposed
  A->>F211: confirmUnfreeze(payer)
  F211-->>A: revert (msg.sender == proposer)
  B->>F211: confirmUnfreeze(payer)
  F211-->>B: frozen[payer] = false
  A->>F210: transferWithMemo(...)
  F210-->>A: status 1 (UC1)
```

Mode A L2 genesis may still have OwnerA = OwnerB (`STATUS.md`). On that env this diagram is **not** true until Sprint 7 / C034.

---

## UC5 — Pause / threshold (two-key params)

Freeze stays one-key. Pause is not freeze.

```mermaid
sequenceDiagram
  autonumber
  actor A as OwnerA
  actor B as OwnerB
  participant F211 as PolicyAdmin
  participant F201 as F201
  participant F210 as F210

  A->>F211: pause()
  F211-->>A: revert (must propose)
  A->>F211: proposePause()
  B->>F211: confirmPause()
  F211-->>B: paused = true
  Note over F210,F201: memos revert (paused reason)
  A->>F211: proposeUnpause()
  B->>F211: confirmUnpause()
```

`setFeeToken` / `setThreshold` same propose+confirm pattern.

---

## UC6 — Fee vs free transfer

Only identified memos pay F213. General-lane `transfer` does not.

```mermaid
sequenceDiagram
  autonumber
  actor P as Payer
  participant F210 as PaymentToken
  participant F213 as PaymasterEntry
  participant Sink as feeRecipient 0x…FEE0

  rect rgb(240,248,255)
    Note over P,Sink: identified (payment lane)
    P->>F210: transferWithMemo
    F210->>F213: takeFee 21000
    F213->>Sink: 21000 F210
    F210->>F210: principal to recipient
  end
  rect rgb(245,245,245)
    Note over P,Sink: general lane
    P->>F210: transfer(to, amt)
    Note over F213: not called
    F210->>F210: principal only
  end
```

---

## UC7 — Payment lane vs high-tip junk

Sprint 3. Lane beats gas price.

```mermaid
sequenceDiagram
  autonumber
  participant Pool as Tx pool
  participant Cls as is_payment
  participant Fill as fill_lanes
  participant Blk as Block

  Pool->>Cls: 20 memos + 20 high-tip transfers
  Cls-->>Fill: payments[] generals[]
  Fill->>Blk: payments first up to 70% gas
  Fill->>Blk: generals up to 30%
  Fill->>Blk: leftover payments then generals
  Note over Blk: high-tip general cannot sit in the first 70% while memos wait
  Note over Blk: if payments empty, general may fill 100%
```

---

## UC8 — Off-us: Quub identity + CCTP USDC to Base (happy)

Sprint 6. Quub is **not** a CCTP domain. Two ledgers, one `endToEndId`.

```mermaid
sequenceDiagram
  autonumber
  actor Orig as Originator
  participant Q as Quub 8091
  participant Gw as quub-gateway
  participant Rec as RailRecord
  participant Sep as Ethereum Sepolia USDC
  participant Cir as Circle attestation
  participant Base as Base Sepolia USDC

  Orig->>Q: transferWithMemo (UC1)  %% identity + policy + fee + evidence
  Q-->>Gw: quubTx + memoHash (status 1)
  Gw->>Gw: F201 would-allow already proven by mined memo
  Gw->>Sep: TokenMessenger.depositForBurn(USDC, destDomain=Base, mintRecipient)
  Sep-->>Gw: message bytes + burnTx
  Gw->>Rec: status = burned
  Gw->>Cir: attest(message)
  Cir-->>Gw: attestation
  Gw->>Base: MessageTransmitter.receiveMessage(message, attestation)
  Base-->>Gw: mintTx  %% USDC arrives on Base
  Gw->>Rec: status = minted<br/>{endToEndId, quubTx, memoHash, burnTx, mintTx}
  Note over Q,Base: burnTx ≠ quubTx. F210 is not USDC.
```

Mock mode: Sep/Cir/Base are in-process fixtures; ids still printed.

---

## UC9 — Freeze blocks the rail

```mermaid
sequenceDiagram
  autonumber
  actor Owner as OwnerA
  actor Orig as Originator
  participant F211 as F211
  participant Q as Quub
  participant Gw as quub-gateway
  participant Sep as CCTP source

  Owner->>F211: freeze(originator)
  Orig->>Q: transferWithMemo
  Q-->>Orig: revert reason 1
  Orig->>Gw: submit rail for that endToEndId
  Gw->>Gw: no mined Quub tx / policy reject fixture
  Gw-->>Orig: no_burn
  Note over Sep: depositForBurn never sent
```

If they try rail-only (skip Quub): **forbidden**. Gateway must require a mined memo (or an eth_call that F201 would allow *and* then mine — year-1 requires the mine first).

---

## UC10 — CCTP fails after Quub already mined

Do not rewind F210. Do not post a second memo.

```mermaid
sequenceDiagram
  autonumber
  actor Orig as Originator
  participant Q as Quub
  participant Gw as quub-gateway
  participant Rec as RailRecord
  participant Sep as CCTP

  Orig->>Q: transferWithMemo
  Q-->>Gw: quubTx mined
  Gw->>Sep: burn
  Sep-->>Gw: revert / timeout
  Gw->>Rec: status = failed (keep quubTx + memoHash)
  Note over Q: F210 balances stay as UC1. No second memo.
  Orig->>Gw: retry same endToEndId
  Gw->>Rec: existing quubTx
  Gw->>Sep: burn again (same identity)
  Sep-->>Gw: burnTx
  Gw->>Rec: burned → minted
```

---

## UC11 — Idempotent replay (API + rail)

```mermaid
sequenceDiagram
  autonumber
  actor Orig as Originator
  participant API as operator-api
  participant Q as Quub
  participant Gw as gateway

  Orig->>API: POST endToEndId=X
  API->>Q: send
  Q-->>API: tx1
  Orig->>API: POST endToEndId=X
  API-->>Orig: 202 tx1 (no tx2)
  Orig->>Gw: rail endToEndId=X
  Gw->>Gw: burn1
  Orig->>Gw: rail endToEndId=X
  Gw-->>Orig: same RailRecord (no burn2)
```

Failed first broadcast is **not** stored as success (Sprint 4 lock). After unfreeze, same `endToEndId` may be retried only if no successful Quub tx exists.

---

## UC12 — On-us vs off-us (same body, `rail` flag)

Planned C048. Default `quub`.

```mermaid
sequenceDiagram
  autonumber
  actor Ops as Ops
  participant API as operator-api
  participant Q as Quub
  participant Gw as gateway
  participant Base as Base USDC

  Ops->>API: POST … rail=quub
  API->>Q: UC1
  API-->>Ops: quubTx, memoHash  (no burn)

  Ops->>API: POST … rail=cctp  (new endToEndId)
  API->>Q: UC1
  API->>Gw: UC8
  API-->>Ops: quubTx, memoHash, burnTx, mintTx
  Base-->>Ops: USDC
```

---

## UC13 — Mode A vs `--dev` (same payment, different consensus)

Not a different payment. Different physical path.

```mermaid
sequenceDiagram
  autonumber
  actor P as Payer
  participant Dev as quub-node --dev :8545
  participant Eng as quub-node --engine :9545
  participant OP as op-node
  participant L1 as geth :8546

  rect rgb(240,248,255)
    Note over P,Dev: --dev
    P->>Dev: transferWithMemo
    Dev->>Dev: local miner + QuubLaneTxs
    Dev-->>P: receipt
  end
  rect rgb(255,248,240)
    Note over P,L1: Mode A
    P->>Eng: transferWithMemo
    Eng->>Eng: pool + QuubLaneTxs
    OP->>Eng: Engine API newPayload (JWT :9551)
    OP->>L1: derive L1 origin
    Eng-->>P: receipt
    Note over OP: kill op-node → L2 head freezes
  end
```

---

## UC14 — Batch disbursement (planned C049)

```mermaid
sequenceDiagram
  autonumber
  actor Ops as Payroll ops
  participant API as POST /v1/payments/batch
  participant Q as Quub lane
  participant F211 as freeze list

  Ops->>API: [row1…row50]
  loop each row
    API->>F211: would this from/to pass?
    alt frozen
      API-->>Ops: that row 422 reason 1
    else
      API->>Q: UC1 or UC2
      Q-->>API: txHash
    end
  end
  Note over Q: 70% lane absorbs the burst vs junk
```

Per-item status, not one atomic revert of the whole file.

---

## UC15 — Travel-rule missing (reason 5)

Field exists today; vendor is later (C045).

```mermaid
sequenceDiagram
  autonumber
  actor Ops as Ops
  participant API as API
  participant F201 as Policy
  participant Vendor as TRP / Sumsub (later)

  Ops->>API: trHash = 0x00… and policy requires TR
  API->>F201: check
  F201-->>API: reason 5 missing_travel_rule
  API-->>Ops: 422 reason 5
  Note over Vendor: later: vendor receipt → trHash; PII stays off-chain
```

---

## Field map (every memo)

| JSON / ISO | On-chain arg | Who sets it |
|---|---|---|
| `to` | `address to` | creditor address book |
| `amount` | `uint256` | minor units of F210 / instruction |
| `endToEndId` | `bytes32` | core / ops — idempotency key |
| `uetr` | `bytes16` | ISO UETR |
| `instrId` | `bytes32` | instruction id |
| `ccy` | `bytes3` | e.g. USD |
| `msgType` | `uint8` | 0 = pacs.008 class |
| `trHash` | `bytes32` | travel-rule commitment |
| `packHash` | `bytes32` | off-chain evidence pack |
| (implicit) | `tx.origin` | signer; inside F202 hash |

---

## What never appears in a payment sequence

- A QUUB ticker buy-in
- `depositForBurn` on chain 8091
- Anvil as SystemConfig L1
- Instant `unfreeze` from the same key
- A second `transferWithMemo` to “fix” a failed CCTP
- EOA calling F201 / F213 `takeFee`
