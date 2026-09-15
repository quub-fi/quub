# Why Quub is needed — by use case
**Claim we will not make:** every banking step requires a new chain.  
**Claim we will make:** once two institutions (or an institution and a public dollar rail) must **share one identified settlement fact**, a private core + a spreadsheet + a correspondent SWIFT message is not enough. Quub is that shared fact: *who paid whom, under which policy, with which evidence*, without issuing a coin and without becoming Circle.

If a step is only inside one bank’s core, Quub is optional. Those rows say so.

---

## The gap Quub is built for

Three systems already exist and still fail to meet each other:

| System | What it is good at | What it cannot be |
|---|---|---|
| Bank core / ISO 20022 | Accounts, limits, messages | A **shared**, append-only settlement record another party can verify |
| Circle CCTP / Base USDC | Moving a dollar that already exists | Policy, freeze, travel-rule hash, ISO identity |
| Public L1s (ETH, Solana, Base) | General execution | A payments desk: reserved block space, maker-checker unfreeze, memo that is not “a comment field” |

Quub is the **policy + identity ledger** that sits on the joint. Dollars stay where they already are (F210 dummy on `--dev`; USDC on Base in production). That is why there is no ticker.

---

## 1. Customer onboarding

**What the bank already does.** KYC, KYB, account opening, risk rating.

**Why people reach for a chain here (and are usually wrong).** “Put identity on-chain.” That puts PII on a ledger. Quub does **not** do that.

**Why Quub is still in the picture.** Onboarding has to produce two artifacts the *payment* path will use later:

1. A **controlled address** (or mapped account) that F211 can freeze.  
2. A **travel-rule profile** whose hash (`trHash`) can be attached to a payment without putting the passport on F212.

Without those, every later payment is “an anonymous ERC-20 transfer.” With them, onboarding is the *enrolment into the policy domain*. Quub is not the KYC vendor. Quub is why the KYC result can **stop a payment on a shared ledger** after the customer is live.

**If you skip Quub.** The bank can still onboard. They cannot freeze that customer’s *settlement* if settlement has already gone to a public USDC transfer the bank does not control.

---

## 2. Beneficiary setup

**Bank already does.** Beneficiary list, name screening.

**Why Quub.** A beneficiary on Base is a public key. Screening in the core does not stop a rogue operator key from paying that wallet on-chain. F211 freeze of the beneficiary address is the control that survives a leaked `QUUB_OPERATOR_KEY` and a CCTP burn.

**If you skip Quub.** Screening is advisory. The dollar can still leave via Circle if anyone has USDC and the destination address.

---

## 3. Payment initiation (customer credit / pain.001)

**Bank already does.** Debit the customer, check funds, sanctions, limits.

**Why Quub.** After the core says “yes,” two other parties still need a **receipt they did not issue**:

- The creditor’s bank (or the platform) needs a memo hash that includes `tx.origin`, UETR, and end-to-end id.  
- Compliance needs evidence (`packHash`) bound to that same hash.  
- The block builder reserves **70%** of the block for that class of transaction so a public mempool of junk cannot delay the wire.

A row in the core is not that receipt. A Swift message is a *copy* of intent; it is not a freeze-able, origin-bound settlement fact on a ledger the other party can verify.

**If you skip Quub.** Initiation still works **on-us** inside one core. The moment you need “the same payment fact on Base USDC *and* in our books,” you are reconciling two worlds by hand.

---

## 4. Request for payment

**Bank already does.** Invoice presentment, RTP / request-to-pay schemes.

**Why Quub is not required to *present* the request.** The request is off-chain. Quub would be theater if we stored invoices.

**Why Quub is needed when the debtor accepts.** Acceptance *is* payment initiation (use case 3). The request id becomes `endToEndId`. Creditor and debtor banks then share one mined memo instead of “we think they paid.”

**If you skip Quub.** RTP still works inside one scheme. Cross-institution proof of *settlement* (especially if cash went out as USDC) is email + bank statement.

---

## 5. Interbank credit transfer (pacs.008)

**What exists.** Swift / correspondent / RTP rails already move messages. Some move money.

**Why Quub.** pacs.008 is a *message*. CCTP is a *burn/mint*. Neither is a **policy checkpoint** that both agents agreed to *before* the dollar minted on Base.

Quub’s job on this use case:

- Refuse the payment if the originator is frozen (sanctions land *before* Circle sees it).  
- Bind UETR + end-to-end id + origin into `memoHash`.  
- Keep that fact even if CCTP is down (retry rail, do not invent a second UETR).

**If you skip Quub.** You can still send pacs.008 and still use CCTP. You cannot prove they are the **same** payment, and you cannot freeze the originator on the rail you do not own.

---

## 6. Payment status

**Bank already does.** Channel status (“processing”).

**Why Quub.** Status after the core is not one bit. It is a **join**:

- Quub receipt = identity recorded.  
- F201 reason = why it died.  
- `RailRecord` = whether the dollar minted.

`GET /v1/payments/{endToEndId}` is that join. Without Quub you have two status machines (core + Circle) and no common key except “amount and time,” which collides.

---

## 7. Cancellation before settlement

**Why Quub is weakly needed.** If nothing was broadcast, the core can cancel alone.

**Why Quub still matters.** Ops must *know* whether the memo mined. That is a chain query, not a guess. After mine, cancel is a lie — you need a return (use case 8). Quub makes the cutover **objective**.

---

## 8. Recall / return / refund

**Bank already does.** camt.056 / pacs.004 processes.

**Why Quub.** A return that is “the opposite row in the core” does not automatically move USDC back or link to the original evidence pack. A **new** memo with `packHash = original memoHash` is the on-chain link. Freeze can block a fraudulent *outbound* while the return is investigated.

**If you skip Quub.** Returns work on-us. Off-us USDC returns are a second manual CCTP and a hope that the reference matches.

---

## 9. Sanctions freeze

**This is the use case that most justifies Quub.**

Cores freeze *accounts*. Public rails freeze *nothing you control* unless you hold the keys to the USDC. Once you have paid toward a public destination, the core freeze is too late.

Quub freeze is:

- Immediate (one officer) — sanctions cannot wait for a second laptop.  
- Enforced in the **execution path** (F201), not a batch job at 2 a.m.  
- Able to block the **next** CCTP burn, not only the next core booking.  
- Reversible only with a **second** officer.

**If you skip Quub.** You have policy in a PDF and settlement on a public chain. Those two will diverge under the first incident.

---

## 10. Travel rule / AML

**Bank already does.** Collect PII, file SAR, talk to a TR network.

**Why Quub.** Regulators and counterparties ask “was travel rule attached to *this* settlement?” A hash on the memo (`trHash`) is the commitment. PII stays in the vendor. The chain answers yes/no without becoming a data store.

**If you skip Quub.** Travel rule is a parallel email. The USDC mint has no binding to the TR packet.

---

## 11. Payroll / bulk disbursement

**Bank already does.** ACH / SEPA files.

**Why Quub.** Two properties ACH does not give a platform that also pays **USDC wallets**:

- A **priority lane** so 500 employee memos are not stuck behind NFT traffic if you share an EVM mempool.  
- A **per-row freeze** (terminated employee) that stops that row’s rail without killing the file.

**If you skip Quub.** Payroll on ACH is fine. Payroll onto Base wallets without a policy ledger is a script and a spreadsheet.

---

## 12. Incoming collection

**Honest.** Year-1 Quub does not watch Base and credit the core. The bank already does collections.

**Why Quub would be added later.** So the collection file and the USDC that arrived share a bank-signed **acknowledgement memo**. Otherwise recon is “we saw a Base tx.” That is not an identified payment; it is a hope.

**If you skip Quub.** Collections still work. Audit is weaker.

---

## 13. Treasury sweep

**Bank already does.** Move money to Coinbase/Circle.

**Why Quub.** The sweep is often the largest payment of the day and the one regulators ask about. A memo + dual-control unfreeze on the treasury address + CCTP record is the difference between “ops moved $40m” and “here is the origin-bound hash and the freeze log.”

**If you skip Quub.** Sweep still works. You cannot prove *which officer’s key* and *which policy state* it left under, except in a SIEM that Circle does not read.

---

## 14. Correspondent payout

**Why this is the strategic use case.** Two banks do not share a core. They may share USDC on Base. They do not share freeze policy.

Quub is the **shared policy domain**:

- Bank A originates with a UETR Bank B already understands.  
- Bank B can later run a **verifier node** and check the memo without trusting A’s screenshot.  
- If A freezes, B does not receive a mint that A’s policy already forbade.

**If you skip Quub.** Correspondents already work (nostro/vostro, Swift). You are choosing **not** to have a joint freeze and a joint evidence hash. That is valid; it is the 1990s shape. Quub is only needed if you want the 8091-shaped joint record.

---

## 15. Marketplace / vendor payout

**Platform already does.** Stripe Connect, Coinbase payouts, etc.

**Why Quub.** Invoice id as `endToEndId`, freeze a vendor under dispute, USDC still lands where vendors already live (Base). The platform does not have to become a bank or a bridge.

**If you skip Quub.** Payouts still work. Dispute + freeze + identitical invoice id on a ledger the vendor can verify does not.

---

## 16. Escrow-style hold

**Why not a new escrow contract.** You already have freeze + two-key release. That *is* the hold.

**Why Quub.** The hold is enforced at **settlement**, not at “please don’t click send.” Releasing requires the second officer. The evidence pack is the contract file (`packHash`).

**If you skip Quub.** Escrow is a lawyer and a suspense account. Fine for one-off. Not a programmable control on a public dollar rail.

---

## 17. Investigation / dispute

**Bank already does.** Case management.

**Why Quub.** The case needs a **packet that does not come from the suspect system**: memoHash (origin-bound), packHash, freeze history, rail ids. That packet is why F212 exists.

**If you skip Quub.** Investigation is logs from the same operator who submitted the payment.

---

## 18. Statement / reconciliation (camt.053 cousin)

**Why Quub.** The statement line is `endToEndId` → Quub tx → optional burn/mint. Without that key you reconcile amount+date and you will mismatch.

**If you skip Quub.** On-us recon is the core vs itself (always “clean”). Off-us recon is the painful one; that is where Quub earns the row.

---

## When Quub is *not* needed

Be explicit with a customer:

| Scenario | Use Quub? |
|---|---|
| One bank, on-us book transfer, no public rail, no second institution | Optional. Core is enough. |
| KYC document storage | No. Vendor. |
| Invoice presentment | No. RTP scheme / AR system. |
| FX conversion | No. Separate engine. |
| Retail “open an app and buy the token” | No. Not the product. |
| Cancel after the memo mined | No. Return payment instead. |
| Replace Swift globally | No. Quub rides next to messages. |
| Replace Circle | No. Circle moves USDC. |

Quub is needed when **at least one** of these is true:

1. A **second party** must verify the payment fact.  
2. Settlement can hit a **public dollar rail** you do not freeze yourself.  
3. Compliance must **stop the next send** in seconds, and **release** only with two keys.  
4. The same id must join **ISO fields + chain receipt + CCTP mint**.

If none of those four are true, do not deploy Quub. That is the product discipline that keeps this from becoming another crowded L1.
