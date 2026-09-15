# Quub — What is implemented later
**Date:** 15 September 2026  
**Now:** Sprints 0–5 accepted. Sprint 6 (CCTP rail) almost done. Sprint 7 (proposed) = API two-key unfreeze, Mode A OwnerB, named datadir, `rail` field on the API.  
**This file:** everything *after* that. Full context so a later Cursor session does not invent a token, a bridge, or a second Reth.

Out of scope (`X001–X010`) is **not** later work. It is refused.

---

## How to read this

Each item has:

- **Why it exists** — the partner problem
- **What already exists** — do not rebuild
- **What to build**
- **Depends on**
- **Acceptance**
- **Do not**

Do not start an item until its depends-on line is true. One ADR per item (or per cluster). Same pins until an ADR changes them as a *set* (`op-node` + `op-reth` + rustc together).

---

## Cluster A — Live dollars (after mock rail is green)

### C040 — Live CCTP ≤ 1 USDC (Sepolia → Base Sepolia)

**Why.** Mock proves the `RailRecord` shape. A partner will ask “did a real dollar move?”

**Already exists.** `CctpAdapter` burn / attest / mint; `specs/rail-cctp.md` with Circle addresses; Quub memo + policy gate; amount cap in the Sprint 6 spec.

**Build.**

- Env: `SEPOLIA_RPC`, `BASE_SEPOLIA_RPC`, `CIRCLE_API` (or Circle’s current attestation URL), `CCTP_BURNER_KEY`.
- Source USDC is **Ethereum Sepolia USDC**, not F210. Dest is **Base Sepolia USDC**.
- Script path already in `rail-base.sh` when those env vars are set. Mode = `live`.
- Cap: **≤ 1 USDC**. Fail if amount > 1e6 (6 decimals) unless a human overrides in a signed config, not a CLI flag someone types twice.

**Depends on.** Sprint 6 mock four-id print; policy-reject never burns; funded Sepolia key with a little ETH + USDC.

**Acceptance.** One run prints Quub tx + memoHash + Sepolia burn tx + Base mint tx. BaseScan shows USDC on the mint recipient. Burn id ≠ Quub tx hash.

**Do not.** Mainnet. Burn QPT. Invent TokenMessenger addresses. Raise the cap “for demo.”

---

### C041 — Ethereum mainnet CCTP (USDC Ethereum ↔ Base)

**Why.** Partner production cash is mainnet USDC, usually Ethereum or Base, not Sepolia.

**Already exists.** Same adapter + `RailRecord` + `endToEndId` idempotency.

**Build.**

- New domain ids + TokenMessenger / MessageTransmitter / USDC from Circle **mainnet** tables (cite URL in `specs/rail-cctp-mainnet.md`).
- Separate env: `ETHEREUM_RPC`, `BASE_RPC`, `CCTP_BURNER_MAINNET_KEY`.
- Amount policy is a **partner config**, not hardcoded 1 USDC. Default still a low cap until dual-control changes it (F211-style: two people to raise the cap).
- Quub chain stays **8091**. You are not launching Quub mainnet as Ethereum.

**Depends on.** C040 live testnet success; KMS for the burner (C038); freeze still blocks burn.

**Acceptance.** One partner-approved amount; three explorers (Quub receipt via your RPC, Etherscan burn, Basescan mint) share one `endToEndId`.

**Do not.** New Quub chain id. Wrapped USDC on 8091. Using anvil keys.

---

### C042 — Solana rail

**Why.** Some originating systems and some destination treasuries are Solana USDC (Circle CCTP supports Solana as a domain).

**Already exists.** `SolanaAdapter` **stub** in `quub-gateway`. Leave it stub until this sprint.

**Build.**

- Solana RPC + Circle Solana TokenMessenger / MessageTransmitter (CCTP domain for Solana — look up, do not guess).
- Signing is **ed25519**, not secp256k1. New key material. Never reuse `QUUB_OPERATOR_KEY`.
- Same `RailRecord` fields. `mint_tx` may be a Solana signature.
- Policy still evaluated on **Quub** before any Solana burn/mint.

**Depends on.** C040 pattern stable; someone who can fund a Solana devnet/mainnet USDC account.

**Acceptance.** Fixture test without network + one devnet or mainnet-dev live tx. Freeze on F211 refuses the Solana call.

**Do not.** Implement Solana inside Sprint 6 leftover. Do not put a Solana VM in `quub-node`.

---

### C043 — CCIP for assets CCTP will not carry

**Why.** USDC has Circle. EURC / other stables / tokenized deposits may need Chainlink CCIP (or a bank’s own rail). Quub still does not become the bridge.

**Already exists.** The sentence in architecture: CCTP primary for USDC, CCIP complementary.

**Build.**

- `CcipAdapter` next to `CctpAdapter`. Same `RailRecord` + `endToEndId`.
- Router / lane addresses from Chainlink docs for **one** pair (e.g. Ethereum ↔ Base) pinned in `specs/rail-ccip.md`.
- Extra: LINK fee on source for CCIP. That fee is **not** F213’s QPT fee. Document both fees.
- Policy gate identical: no CCIP send if Quub would reject.

**Depends on.** C040 done so you do not invent a second pattern. Partner names the asset.

**Acceptance.** Fixture + optional testnet CCIP message id stored on F212 `packHash` or in `RailRecord`.

**Do not.** Rebuild USDC transfer on CCIP “for unity.” Do not add a Quub-wrapped version of the asset.

---

## Cluster B — Operate it like a service

### C036 — TLS + `api.quub.network` / `rpc.quub.network`

**Why.** `127.0.0.1:8080` is not a partner endpoint. You already own `quub.network` and `quub.fi`.

**Already exists.** operator-api on loopback; `/health` fail-closed on chain id ≠ 8091.

**Build.**

- DNS: `api.quub.network` → API host; `rpc.quub.network` → `eth_*` only (or API-only first, RPC private).
- `quub.fi` stays marketing. Do not put JSON-RPC on `.fi`.
- TLS terminate at an edge (Caddy/nginx). App still binds `127.0.0.1`.
- No `0.0.0.0` on `quub-node` or `operator-api` even then.

**Depends on.** A VM that is not the laptop (see `PHYSICAL.md` four-host split); certs (Let’s Encrypt is enough for test).

**Acceptance.** `curl https://api.quub.network/health` returns 8091. HTTP on 80 redirects. Engine `:9551` is **not** in public DNS.

**Do not.** Publish `:9551` or geth `:8546`. Do not use anvil keys behind public TLS.

---

### C037 — Postgres idempotency

**Why.** Process-local `DashMap` dies on restart. A bank will POST the same `endToEndId` again after your API bounced.

**Already exists.** In-memory map keyed by `endToEndId`; 409/202 replay of the first `txHash`.

**Build.**

- Table `payments(end_to_end_id PK, tx_hash, status, memo_hash, rail_status, created_at, updated_at)`.
- Same uniqueness for rail so CCTP does not double-burn across API restarts.
- Local Docker Postgres is enough. Connection string `QUUB_DATABASE_URL`.
- Migration in-repo. No SQLite “for now” if you will throw it away in a week — pick Postgres and stay.

**Depends on.** Sprint 4 API stable; C033 two-key unfreeze merged so you do not persist a broken unfreeze path.

**Acceptance.** Start API, pay, kill API, start API, POST same id → same `txHash`, no second Quub tx.

**Do not.** Store failed policy rejects as successful rows (Sprint 4 lock).

---

### C035 leftover — Named datadirs in every script

**Why.** `--dev` ephemeral datadir makes yesterday’s receipt hash evaporate. Demos then “fail.”

**Build.** `QUUB_DATADIR` default `./artifacts/datadir-dev` and `./artifacts/datadir-mode-a`. Scripts must not wipe unless `QUUB_RESET=1`.

**Depends on.** Nothing. Can ride with Sprint 7.

**Acceptance.** Restart `--dev`, `cast receipt $HASH` still works.

---

### C046 — Regulator / ops extract

**Why.** Compliance will not grep logs. They want a period file: who paid whom, freeze list, rail ids.

**Already exists.** Events `MemoAnchored`, `EvidenceAnchored`, F211 freeze storage, `RailRecord` JSON.

**Build.**

- `scripts/extract.sh --from-block --to-block --out extract.csv`
- Columns: block, tx, from, to, amount, endToEndId, uetr, memoHash, packHash, freeze state at block, rail burn/mint if any.
- Read-only against `QUUB_RPC`. No owner key.

**Depends on.** C037 helps for API-side fields; chain events are enough for v1.

**Acceptance.** One CSV that matches a known `sprint5-dev` payment + a freeze.

**Do not.** Build a full explorer product in this item.

---

### C038 — KMS / split keys

**Why.** `PHYSICAL.md`: the machine that freezes must not hold the CCTP burner. Anvil keys are public.

**Already exists.** OwnerA / OwnerB roles; JWT file; `QUUB_OPERATOR_KEY` env.

**Build.**

- OwnerA, OwnerB, operator signer, sequencer JWT, CCTP burner = **five** materials.
- Year-2: cloud KMS (AWS KMS / GCP KMS / CloudHSM) or two hardware wallets for owners.
- API host signs memos. Sequencer host holds JWT. Rail host holds burner. Owners confirm unfreeze from two laptops / two HSMs.
- Rotate runbook. Anvil keys deleted from scripts behind `QUUB_ALLOW_ANVIL=1`.

**Depends on.** Four-host split or at least two hosts; C036 so you are not encrypting loopback theater.

**Acceptance.** A payment and a freeze in staging with **zero** anvil keys in env. Unfreeze still needs two distinct KMS keys.

**Do not.** One KMS key used for everything. Do not put OwnerB in the API process.

---

## Cluster C — Public settlement (leave the laptop L1)

### C039 — op-batcher + op-proposer to a public L1

**Why.** Mode A today settles to **local geth**. Nobody outside the laptop can verify 8091. A partner’s auditor will ask.

**Already exists.** Mode A wiring: `quub-node --engine`, `op-node` v1.19.7, `op-deployer` genesis, SystemConfig `0xe991…F990` on local L1. Kill-test.

**Build.**

- New env `mode-a-sepolia`: L1 = public Sepolia, not `127.0.0.1:8546`.
- Run `op-batcher` + `op-proposer` versions paired with `op-node` v1.19.7 (or bump **the whole OP set** in one ADR).
- Fund batcher/proposer Sepolia ETH.
- Quub L2 genesis still etches F210–F213. Chain id stays 8091 unless Sepolia Superchain politics force a registered id — that is a **separate ADR**, not a silent change.
- Verifier host: `quub-node` + `op-node` **without** sequencer key (C047).

**Depends on.** C038 at least for sequencer JWT; C034 OwnerB in L2 genesis; a host that stays on.

**Acceptance.** A stranger with Sepolia RPC + your rollup.json can follow L2 head. Kill sequencer → unsafe head stops; verifier still reads posted batches.

**Do not.** Anvil L1. Register Superchain before keys are in KMS. Bump only Reth.

---

### C047 — Read-only verifier node

**Why.** Bank B or a regulator should check memos without trusting your sequencer.

**Already exists.** Same binary. Mode A already requires `op-node` to derive.

**Build.** Config profile `quub-node --engine --verifier` (or documented flags) that never loads the sequencer JWT. Docs: “here is rollup.json, here is L1, here are F2xx addresses.”

**Depends on.** C039 (otherwise they can only verify your laptop).

**Acceptance.** Verifier head matches sequencer after a memo. Verifier cannot `freeze`.

---

## Cluster D — Bank origination (the actual product)

### C044 — pacs.008 in

**Why.** Banks do not speak `transferWithMemo`. They speak ISO 20022. Fields on F210 already match a stub.

**Already exists.** `quub-iso` validates empty endToEndId / UETR / ccy; operator-api JSON body.

**Build.**

- Ingest: file drop, SFTP, or MQ (partner picks). Parse pacs.008.
- Map: `EndToEndId` → `endToEndId`, `UETR` → `uetr`, `InstrId` → `instrId`, currency, amount (minor units), creditor account → `to` (needs an account→address book).
- `trHash` from C045 or zero with reason 5 if policy requires it.
- Out: existing `POST /v1/payments` or in-process call. Do not add a second payment path in the node.

**Depends on.** A design partner’s sample pacs.008. Address book. C037 so duplicates from the core do not double-send.

**Acceptance.** One real (sanitized) pacs.008 file → mined memo whose `endToEndId` equals the XML.

**Do not.** Write a full ISO engine for every message type. pacs.008 first. camt.053 later if they ask.

---

### C045 — Travel-rule vendor hash

**Why.** Reason 5 (`missing_travel_rule`) is already in policy. The hash is a fixture (`0x44…`).

**Build.**

- Plug one vendor (TRP / Sumsub / Sygna / Notabene — partner picks).
- API: given originator + beneficiary + amount → `trHash`. Store the vendor receipt off-chain; put the hash on chain.
- If vendor down: fail closed if policy requires TR; do not silently send.

**Depends on.** Vendor contract + sandbox keys. C044 optional but usually the same project.

**Acceptance.** Missing vendor hash → 422 reason 5. Good hash → memo mines. Fixture `0x00…` rejected when TR required.

**Do not.** Invent a travel-rule protocol. Do not put PII on F212.

---

### C048 — On-us vs off-us (API `rail` field)

**Why.** Same payment body. On-us stops at Quub. Off-us adds CCTP.

**Already exists.** Two code paths: F210 memo; `CctpAdapter`. Sprint 6 is a script.

**Build.** `POST /v1/payments` optional `"rail": "quub" | "cctp" | "ccip" | "solana"`. Default `quub`. After mine, if rail ≠ quub, call gateway. Return all ids.

**Depends on.** Sprint 6 crate; C033 not strictly required but do not ship public API with broken unfreeze.

**Acceptance.** Two curls, same body, different `rail`; only `cctp` grows burn/mint ids.

**Do not.** A second HTTP service that bypasses F201.

---

### C049 — Disbursement / payroll batch

**Why.** 70% lane exists so a burst of memos beats junk traffic. Payroll is that burst.

**Build.** `POST /v1/payments/batch` (cap e.g. 100). Each item is an existing payment body. Fail-one vs fail-all is a **written** choice (recommend per-item status, not atomic). Freeze list applied per row.

**Depends on.** C016 API + C010 lane + C037.

**Acceptance.** 50 memos in one request; lane flood still ≥65% payment share; one frozen row 422, others mine.

**Do not.** A new tx type. Do not disable fees unless ADR.

---

### C050 — Marketplace vendor payout

**Why.** Platform already pays vendors; vendors already hold USDC on Base.

**Build.** Product config: platform operator key + vendor address book + default `rail=cctp`. Mostly C048 + C041. Almost no new chain work.

**Depends on.** Live CCTP (C040/C041) + partner.

**Acceptance.** One vendor receives Base USDC; `endToEndId` matches the platform invoice id.

**Do not.** Build a marketplace.

---

## Cluster E — Security / control leftovers

### C033 — API two-key unfreeze  
Sprint 7. Context: Sprint 4 API calls `unfreeze` as if it were still one send. Sprint 5 removed that. Either the API drives propose+confirm with `QUUB_OWNER_B_KEY`, or unfreeze stays CLI-only (`sprint5-dev.sh`). Do not leave a 200 that never unfreezes.

### C034 — Mode A OwnerB in L2 genesis  
Sprint 7. Context: `--dev` alloc is anvil0/anvil1. `l2-genesis-quub.json` from op-deployer may still be one owner. Dual-control on 9545 is documentation until the slot is etched. Regen with op-deployer merge; do not hand-edit SystemConfig.

---

## Order after Sprint 6 (recommended)

```
7    C033 C034 C035 C048          (this repo, this week)
8    C037 + C046                  (Docker Postgres + extract CSV)
9    C040                         (live ≤1 USDC — env gated)
10   C036 + four-host             (TLS; not a crate-only sprint)
11   C038                         (KMS; blocks mainnet)
12   C039 + C047                  (public L1 + verifier)
13   C044 + C045                  (bank file + TR vendor — needs partner)
14   C041 / C042 / C043           (one rail per sprint)
15   C049 C050                    (product APIs on top)
```

Skip 10–12 if you only ever demo on a laptop. Do **not** skip 7. Do **not** jump to 14 before 9.

---

## Still never

Native ticker. Simplex in the Mode A binary. Quub-minted USDC. Anvil as L1. Second Reth remote. Superchain registry before KMS. zkVM as a year-1 milestone. DEX / NFT / retail wallet as the pitch. `0.0.0.0` on the node.
