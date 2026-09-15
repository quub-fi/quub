# Quub — Business payment use cases
**Audience:** product, banks, ops.  
**Ledger under the hood:** Quub 8091 records *who paid whom and why*. Dollars that leave the bank move as **USDC on Base via Circle CCTP**. Quub does not mint dollars and does not onboard retail wallets as the year-1 product.

Status: **now** = API + node you have. **rail** = Sprint 6. **later** = `LATER.md`.

---

## 1. Customer onboarding (institution, not retail)

A bank or PSP enrols a *customer* (corporate or correspondent) so they may originate or receive identified payments.

**Now:** address book is manual (`to` is a hex address). Freeze list is F211.  
**Later:** KYC case id stored off-chain; `trHash` from a vendor; account→address map in the API.

```mermaid
sequenceDiagram
  autonumber
  actor Bank as Bank onboarding
  actor Cust as Customer (corporate)
  participant KYC as KYC / KYB vendor
  participant Book as Account book (API / core)
  participant F211 as Quub freeze list
  participant TR as Travel-rule vendor (later)

  Cust->>Bank: apply (legal name, accounts, jurisdictions)
  Bank->>KYC: KYB + sanctions screen
  alt fail
    KYC-->>Bank: reject
    Bank-->>Cust: not onboarded
  else pass
    KYC-->>Bank: case id
    Bank->>Book: map IBAN / account → Quub address / Base USDC address
    Bank->>TR: create originator profile (later)
    Note over F211: default not frozen
    Bank-->>Cust: live for origination / receipt
  end
```

**Quub does not:** host the KYC documents, issue deposit accounts, or run the screening engine.

---

## 2. Beneficiary setup

Customer registers who they are allowed to pay.

```mermaid
sequenceDiagram
  autonumber
  actor Cust as Customer
  actor Bank as Bank ops
  participant Book as Beneficiary book
  participant Screen as Sanctions
  participant F211 as Freeze list

  Cust->>Bank: add beneficiary (name, account, Base USDC wallet)
  Bank->>Screen: screen name + wallet
  alt hit
    Bank->>F211: freeze(wallet) optional
    Bank-->>Cust: rejected
  else clear
    Book-->>Cust: beneficiary id
  end
```

---

## 3. Payment initiation (customer → bank → Quub)

pain.001 / “send money.” Year-1 entry is JSON to `POST /v1/payments`. Later that JSON is filled from pain.001 / pacs.008.

```mermaid
sequenceDiagram
  autonumber
  actor Cust as Customer
  participant Core as Bank core / channels
  participant API as Quub operator-api
  participant Q as Quub 8091
  participant Rail as CCTP (if off-us)

  Cust->>Core: initiate payment (debtor, creditor, amount, purpose, e2e id)
  Core->>Core: funds check + limits + sanctions
  Core->>API: POST /v1/payments {to, amount, endToEndId, uetr, instrId, ccy, …}
  API->>Q: transferWithMemo
  alt on-us (creditor on this bank)
    Q-->>API: mined + memoHash
    API-->>Core: accepted / settled_on_quub
    Core-->>Cust: completed
  else off-us (creditor wants USDC on Base)
    Q-->>API: mined + memoHash
    API->>Rail: burn Sepolia/ETH USDC → mint Base
    Rail-->>API: burnTx + mintTx
    API-->>Core: completed + rail ids
    Core-->>Cust: completed
  end
```

**Idempotency:** same `endToEndId` never creates a second Quub payment.

---

## 4. Request for payment / request to pay

Creditor asks debtor to pay. Quub does not store invoices. The bank does. When the debtor accepts, it becomes use case 3 with the *creditor’s* `endToEndId`.

```mermaid
sequenceDiagram
  autonumber
  actor Cred as Creditor customer
  actor Debt as Debtor customer
  participant Core as Bank / RTP service
  participant API as Quub API
  participant Q as Quub

  Cred->>Core: request to pay (amount, due, invoice, debtor)
  Core-->>Debt: request presented
  alt refuse
    Debt->>Core: reject
    Core-->>Cred: refused
  else accept
    Debt->>Core: accept
    Core->>API: POST /v1/payments (endToEndId = request id)
    API->>Q: transferWithMemo
    Q-->>Core: mined
    Core-->>Cred: paid
    Core-->>Debt: debit confirmation
  end
```

**Now:** request lives in the bank. Quub only sees the resulting payment.  
**Later:** optional `msgType` for RTP vs credit transfer (field already on the memo).

---

## 5. Credit transfer (pacs.008) — interbank

Correspondent or scheme-style credit. Same on-chain memo; different origination.

```mermaid
sequenceDiagram
  autonumber
  participant BankA as Debtor agent
  participant BankB as Creditor agent
  participant API as Quub API
  participant Q as Quub
  participant CCTP as Circle
  participant Base as Base USDC

  BankA->>API: pacs.008 mapped fields (UETR required)
  API->>Q: memo
  Q-->>API: memoHash
  opt off-us cash
    API->>CCTP: USDC to BankB’s Base wallet
    CCTP->>Base: mint
  end
  API-->>BankB: pacs.002-like status + ids
```

**Now:** JSON body is the pacs.008 field subset.  
**Later:** ingest real pacs.008 XML (C044).

---

## 6. Payment status (pacs.002 / customer status)

```mermaid
sequenceDiagram
  autonumber
  actor Cust as Customer
  participant Core as Bank
  participant API as GET /v1/payments/{endToEndId}
  participant Q as Quub
  participant Rec as RailRecord

  Cust->>Core: where is payment X?
  Core->>API: GET X
  API->>Q: receipt
  API->>Rec: rail status
  API-->>Core: submitted | mined | failed | rejected | burned | minted
  Core-->>Cust: pending / completed / failed + bank reason
```

Map: Quub `reason 1` → “blocked by compliance.” CCTP `failed` → “settlement delayed, identity already recorded.”

---

## 7. Cancellation / stop before settlement

If the Quub tx is **not** mined, the API simply does not retry. If it **is** mined, Quub does not rewind. Stop-pay after mine = freeze + off-chain refund process, or a **new** opposite payment (use case 9).

```mermaid
sequenceDiagram
  autonumber
  actor Cust as Customer
  participant Core as Bank
  participant API as Quub API
  participant Q as Quub

  Cust->>Core: cancel endToEndId X
  Core->>API: GET X
  alt not broadcast / not mined
    Core-->>Cust: cancelled
  else mined
    Core-->>Cust: too late on Quub — use recall / return
  end
```

---

## 8. Recall / return (after the money moved)

A new identified payment the other way. Same original `endToEndId` in `packHash` or a linked field — do not reuse the original id as the new payment’s id.

```mermaid
sequenceDiagram
  autonumber
  actor BankB as Creditor bank
  actor BankA as Debtor bank
  participant API as Quub API
  participant Q as Quub
  participant CCTP as CCTP

  BankB->>BankA: recall (wrong account / fraud)
  BankA->>API: POST new payment (return) + packHash = original memoHash
  API->>Q: transferWithMemo (new e2e id)
  opt original was off-us
    API->>CCTP: USDC back to BankA Base/ETH wallet
  end
  API-->>BankB: return mined
```

---

## 9. Refund (merchant / platform)

Same as return; initiator is the merchant’s bank. Invoice id = `endToEndId` of the refund, `packHash` = original sale.

---

## 10. Sanctions / freeze (compliance hold)

Customer or beneficiary hits a list. Payments desk freezes **immediately**. Release needs two officers.

```mermaid
sequenceDiagram
  autonumber
  actor CO as Compliance officer A
  actor CO2 as Officer B
  actor Cust as Customer
  participant API as POST /v1/freeze
  participant F211 as PolicyAdmin
  participant Q as Quub

  CO->>API: freeze(customer address)
  API->>F211: freeze
  Cust->>API: initiate payment
  API-->>Cust: 422 frozen
  Note over CO2: investigation off-chain
  CO->>F211: proposeUnfreeze
  CO2->>F211: confirmUnfreeze
  Cust->>API: initiate payment
  API-->>Cust: mined
```

---

## 11. Travel-rule / AML attach

**Now:** `trHash` on the payment; missing hash can be reason 5.  
**Later:** vendor produces the hash; PII never goes on Quub.

```mermaid
sequenceDiagram
  autonumber
  actor Cust as Customer
  participant Bank as Bank
  participant TR as Travel-rule network
  participant API as Quub API

  Cust->>Bank: pay (originator + beneficiary PII)
  Bank->>TR: submit PII + amount
  TR-->>Bank: trHash
  Bank->>API: POST payment + trHash
  API-->>Bank: mined
```

---

## 12. Payroll / bulk disbursement

```mermaid
sequenceDiagram
  autonumber
  actor HR as Payroll
  participant Core as Bank
  participant API as Quub API (batch later)
  participant Q as Quub payment lane
  participant Base as Employee USDC wallets

  HR->>Core: payroll file
  Core->>Core: drop frozen employees
  loop each row
    Core->>API: payment (e2e = employee+period)
    API->>Q: memo (70% lane)
    opt wage rail
      API->>Base: USDC
    end
  end
  Core-->>HR: accepted / rejected per row
```

---

## 13. Collections / incoming from another chain

Someone already has USDC on Base and wants it recognized as a collection against an invoice. Year-1 Quub is **originate-then-rail**, not “watch Base and mint on Quub.” Incoming is later: observe mint, then post a Quub memo as the bank’s acknowledgement.

```mermaid
sequenceDiagram
  autonumber
  actor Payer as External payer
  participant Base as Base USDC
  participant Bank as Creditor bank
  participant API as Quub API
  participant Q as Quub

  Payer->>Base: USDC to bank wallet (outside Quub)
  Bank->>Bank: match amount + reference
  Bank->>API: POST acknowledgement memo (msgType = incoming)
  API->>Q: transferWithMemo (bank internal)
  Q-->>Bank: evidence for the collection file
```

Do not treat a random Base transfer as a Quub settlement without the bank’s memo.

---

## 14. Treasury sweep

Bank moves house funds to a Base treasury wallet it already uses with Coinbase/Circle.

```mermaid
sequenceDiagram
  autonumber
  actor Treas as Treasurer
  participant API as Quub API
  participant Q as Quub
  participant CCTP as CCTP
  participant TreasW as Base treasury wallet

  Treas->>API: POST (to = sweep address, rail = cctp)
  API->>Q: identity + policy
  API->>CCTP: USDC to TreasW
  CCTP-->>Treas: minted
```

---

## 15. Correspondent payout

Bank A originates; Bank B’s customer receives USDC on Base (or stays on-us if both are on Quub).

```mermaid
sequenceDiagram
  autonumber
  actor CA as Bank A customer
  participant A as Bank A
  participant Q as Quub
  participant B as Bank B
  participant Base as Bank B Base wallet

  CA->>A: pay B’s customer
  A->>Q: memo (UETR shared with B)
  A->>Base: CCTP to B
  B-->>B customer: credit per B’s books
```

Quub is the shared identity spine. B does not have to run a Quub sequencer; B can be a verifier later (C047) or just take USDC + the `memoHash` in a message from A.

---

## 16. Invoice marketplace / vendor payout

Platform pays vendors. Invoice id = `endToEndId`. Vendor already has a Base address.

```mermaid
sequenceDiagram
  autonumber
  actor Plat as Platform
  actor Ven as Vendor
  participant API as Quub
  participant Base as Vendor USDC

  Plat->>API: pay vendor (e2e = invoice)
  API->>API: policy + memo
  API->>Base: USDC
  API-->>Plat: ids for the invoice
  Ven-->>Ven: sees USDC on Base
```

---

## 17. Escrow-like hold (policy, not an escrow VM)

Hold = freeze the payer or the payout address until two officers release. Not a smart-escrow product.

```mermaid
sequenceDiagram
  autonumber
  actor Desk as Payments desk
  actor Comp as Second officer
  participant F211 as Freeze
  participant API as Quub API

  Desk->>F211: freeze(payout address)
  Note over API: payments to that address fail
  Desk->>Comp: documents ok
  Desk->>F211: proposeUnfreeze
  Comp->>F211: confirm
  Desk->>API: release payment (UC3)
```

---

## 18. Investigation / dispute file

Ops pulls one `endToEndId` and hands compliance a pack.

```mermaid
sequenceDiagram
  autonumber
  actor Ops as Ops
  participant API as GET payment
  participant Q as Quub events
  participant Rec as RailRecord
  participant Files as Off-chain pack (packHash)

  Ops->>API: endToEndId
  API->>Q: memoHash, from, to, amount, origin
  API->>Rec: burn/mint if any
  Ops->>Files: retrieve packHash
  Ops-->>Compliance: one folder
```

---

## What Quub is not (so these are not use cases)

| Not a use case | Why |
|---|---|
| Open a deposit account | Bank core |
| Card acquiring | Different rail |
| FX engine | Amount is already in one `ccy` |
| Retail self-custody wallet | Year-1 actor is the bank operator |
| “Buy QUUB to pay” | No network token |
| Instant cancel after mine | Immutable memo; use return |

---

## Mapping to what you already shipped

| Business use case | Today | Missing |
|---|---|---|
| Onboarding | Manual address + freeze | KYC vendor, account book |
| Beneficiary | Manual `to` | Screening workflow |
| Initiation | `POST /v1/payments` | pain.001 file |
| Request to pay | Off-chain request | `msgType` convention |
| Credit transfer | Memo fields | pacs.008 XML |
| Status | `GET /v1/payments/{id}` | pacs.002 out |
| Cancel before mine | Don’t send | Explicit cancel API |
| Return / refund | New memo | Link field standard |
| Freeze / unfreeze | API + two-key unfreeze | API two-key wired |
| Travel rule | `trHash` slot | Vendor |
| Payroll | Loop + 70% lane | Batch endpoint |
| Off-us USDC | Sprint 6 rail | Live CCTP keys |
| Incoming collection | Not built | Watch + ack memo |
| Investigation | Events + GET | Extract CSV |
