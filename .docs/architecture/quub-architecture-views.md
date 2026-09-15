# Quub — Architecture views
**Product:** Quub, a payments blockchain  
**Date:** 15 September 2026  
**Rule:** one product, no native token, no second brand  
**Chain id:** 8091 on every Quub execution path  
**Pins:** `op-rs/reth` rev `aef8d3ef92117f91455e16969f0adf5bf7c6e9e1`, optimism `op-reth/v2.4.4`, `op-node` v1.19.7, rustc 1.96

Four views. Same system. Different altitude.

1. **Ecosystem** — where Quub sits among rails that already exist  
2. **System** — the Quub parts and who talks to whom  
3. **Implementation** — how those parts are built  
4. **Deployment** — what runs where  

Later work (TLS, KMS, public batcher, Solana, CCIP, pacs.008 XML, Mode B) lives in `LATER.md`. It is not drawn as if it were live.

---

## View 1 — Ecosystem (where Quub plays)

Quub does not replace SWIFT, Solana, Base, or a bank core. It is a **policy and identity ledger** a licensed institution can contract, with an exit onto the rail the counterparty already uses.

Year-1 exit is **Circle CCTP → Base USDC**. Other public rails are later adapters on the same `RailRecord`.

```
                         VALUE MOVES HERE
  USDC (Circle) — year-1
  other stables / other chains — later
           │
           │  native issuance (Circle, later other issuers)
           ▼
 ┌─────────┴──────────┬─────────────┬──────────────┐
 │ Public payment     │ Quub        │ Bank / RTGS  │
 │ rails              │ 8091        │ messaging    │
 │                    │             │              │
 │ Base     USDC      │ Policy F201 │ SWIFT CBPR+  │
 │ Ethereum USDC      │ Memo   F202 │ core / ISO   │
 │ (Solana later)     │ Fee    F213 │              │
 │                    │ Lane   70%  │              │
 │                    │ Evidence    │              │
 └─────────┬──────────┴──────┬──────┴──────┬───────┘
           │                 │             │
           └────────────┬────┴─────────────┘
                        ▼
              operator-api  :8080
              quub-gateway  (rail)
                        ▲
                        │
              PSP / bank / EMI / ops
```

**What already exists and we do not rebuild**

| Layer | Who owns it | Quub’s relationship |
|---|---|---|
| Dollar inventory | Circle USDC (year-1) | Native paper. Never wrap as the unit of record. F210 on `--dev` is QPT dummy, not USDC |
| Public cheap rails | Base first; Ethereum CCTP source; Solana later | Gateway exits via CCTP. Quub 8091 is **not** a CCTP domain |
| Bank messaging | SWIFT / core | API ingests a JSON subset of ISO fields. Full XML is later |
| Custody | Fireblocks / bank custody later | Year-1 is `eth_*` + operator key. Standard JSON-RPC |
| Travel rule | Vendor later | Field `trHash` already exists. Vendor writes the hash. Quub checks it |
| Identity / KYC | the licensed client | Quub does not become a KYC vendor |

**Where Quub is the system of record (year-1)**

For an **identified** payment:

- freeze / allow cannot be skipped by calling the token a different way  
- ISO identity is committed on the receipt (`memoHash` includes `tx.origin`)  
- a listed-token fee is taken only on `transferWithMemo`  
- mempool junk cannot consume the payment lane (70%)  
- evidence hash is on-chain; the file is off-chain  

Year-1 off-us path is **Quub memo first, then CCTP**. Policy reject ⇒ no burn. The chain is not optional for that path.

A future “gateway-only, funds never touch 8091” topology needs its own ADR. It is not current behavior.

**What we charge for**

Orchestration and settlement service: API / gateway fees, later corridor FX, compliance pack. Sequencer fees in a listed stable if ever. Not a ticker.

---

## View 2 — System (components and contracts)

```
                    Client systems
             ERP · PSP core · treasury · ops
                         │
                         │  REST (pain.001 / pacs.008 XML later)
                         ▼
              ┌─────────────────────┐
              │  operator-api :8080 │
              │  Bearer · ISO check │
              └──────────┬──────────┘
                         │ eth_*
           ┌─────────────┼──────────────┐
           ▼             ▼              ▼
      quub-iso     quub-policy     quub-gateway
      field map    reason codes    CCTP rail
           │             │              │
           └─────────────┼──────────────┘
                         ▼
              ┌─────────────────────┐
              │     quub-node       │
              │   --dev    :8545    │
              │   --engine :9545    │
              │   eth_* JSON-RPC    │
              └──────────┬──────────┘
                         │
         ┌───────────────┼────────────────┐
         ▼               ▼                ▼
   Execution         Consensus         Data
   QuubEvmFactory    --dev miner       --dev: local
   F201 Policy       Mode A:           Mode A: local geth L1
   F202 Memo           op-node v1.19.7   (public L1 later)
   F203 thin           Engine API
   F210 Token        Mode B Simplex:
   F211 PolicyAdmin    not year-1
   F212 Anchor
   F213 Paymaster
   Lane 70/30
```

### On-chain (consensus-critical)

| Component | Address | Responsibility | Status |
|---|---|---|---|
| Quub Policy | `F201` | Stateful `check`. Caller must be F210. Sloads F211 | Shipped |
| Quub Memo | `F202` | Hash of ISO identity + `tx.origin`. No PII | Shipped |
| Quub Paymaster precompile | `F203` | Thin. Does not move funds | Shipped stub |
| Quub Token | `F210` | Payment asset on 8091. `--dev` = QPT dummy | Shipped |
| Quub Policy Admin | `F211` | Freeze immediate (either owner). Unfreeze / params = other owner | Shipped. No 24h timelock |
| Quub Anchor | `F212` | `packHash` + memoHash. Not the file | Shipped |
| Quub Fee Entry | `F213` | `quote` + `takeFee` → `paymasterDebit`. Only F210 may call | Shipped |
| Quub Lane | pool + payload | 70% block gas for exact `transferWithMemo` on F210 | Shipped |

### Node-adjacent

| Component | Responsibility | Status |
|---|---|---|
| operator-api | HTTP door: pay, status, freeze. Bind `127.0.0.1:8080` | Shipped |
| quub-gateway | CCTP adapter + `RailRecord` by `endToEndId` | Sprint 6 |
| quub-iso | Empty id / UETR / ccy checks | Shipped stub |
| quub-policy | Reason codes | Shipped |
| Evidence file store | Object store for `packHash` | Later. F212 is hash only |

### External (year-1 vs later)

- Circle CCTP V2 — year-1 (mock valid; live ≤ 1 USDC when env set)  
- Base Sepolia / Base — year-1 destination  
- Ethereum Sepolia — year-1 CCTP **source** (not 8091)  
- Solana, CCIP, Tempo, Arc, Tron — later rails  
- Travel-rule vendor — later (`trHash` slot exists)  
- HSM / Fireblocks — later  

### What a year-1 payment looks like

```
Ops / core          operator-api         quub-node              quub-gateway         CCTP / Base
  │                      │                    │                      │                    │
  │  POST /v1/payments   │                    │                      │                    │
  │─────────────────────►│                    │                      │                    │
  │                      │  iso + idempotency │                      │                    │
  │                      │  transferWithMemo  │                      │                    │
  │                      │───────────────────►│                      │                    │
  │                      │                    │ F201 allow           │                    │
  │                      │                    │ F213 fee (memo only) │                    │
  │                      │                    │ F210 principal       │                    │
  │                      │                    │ F202 memoHash        │                    │
  │                      │                    │ F212 packHash        │                    │
  │                      │◄─ mined + memoHash─┤                      │                    │
  │                      │                    │                      │                    │
  │                      │  if rail = cctp    │                      │                    │
  │                      │──────────────────────────────────────────►│                    │
  │                      │                    │                      │  burn source USDC  │
  │                      │                    │                      │───────────────────►│
  │                      │                    │                      │  attest + mint     │
  │                      │                    │                      │───────────────────►│
  │  GET {endToEndId}    │                    │                      │                    │
  │◄─ quubTx, memoHash, burn, mint ───────────┴──────────────────────┴────────────────────┘
```

If F201 rejects: no fee, no principal, no burn.

---

## View 3 — Implementation (how it is built)

### Repository

```
quub.network/
  crates/
    quub-primitives      # addresses, reason codes, PAYMENT_LANE_BPS=7000
    quub-precompiles     # F201/F202/F203 bodies
    quub-evm             # QuubEvmFactory + wrap + slots
    quub-pool            # is_payment: F210 + exact transferWithMemo
    quub-payload         # fill_lanes 70/30
    quub-node            # --dev and --engine
    quub-consensus-op    # Mode A (Sprint 2). ADR-016 pin
    quub-consensus-simplex   # stub. feature off
    quub-rpc             # eth_* only
  apps/
    operator-api         # :8080
  services/
    quub-iso
    quub-policy
    quub-gateway         # CCTP
    quub-evidence        # hash helpers; file store later
  contracts/             # Foundry: PaymentToken, PolicyAdmin, EvidenceAnchor, PaymasterEntry
  specs/
  docs/adr/              # 016 pin, 017 lane, 019 fee+dual-control, 020 rail
```

### Build rules

- Node language: Rust. Money contracts: Solidity + Foundry.  
- **One** Reth remote: `op-rs/reth@aef8d3ef`. Do not add `paradigmxyz/reth`. Do not float main.  
- OP crates from `ethereum-optimism/optimism` tag `op-reth/v2.4.4`.  
- rustc **1.96.0** (1.95 failed `vergen` on this pin).  
- Precompile inject when spec ≥ PRAGUE (or OP equivalent, same intent). F201/F202 `DynPrecompile::new_stateful`.  
- Policy engine and F201 share reason codes. F201 is the enforcement.  
- JSON-RPC is `eth_*`. No `quub_` namespace year-1.  
- No native token. `--dev` ETH is miner fuel, not the product.  
- No new EIP-2718 type. Classify existing txs for the lane.  
- Mode A and Mode B never compile into the same binary under test.

### Stack by layer

```
 Application     Foundry contracts
                 operator-api (Axum) + Alloy against QUUB_RPC
                 quub-iso / quub-gateway

 Execution       quub-evm → op-rs/reth@aef8d3ef → REVM
                 F201 F202 stateful, F203 thin
                 F210–F213 genesis etch (never CREATE onto F210)

 Fee             F213 takeFee in listed token (QPT on --dev)
                 not 1559, not ETH gas, not a ticker

 Consensus       --dev: local miner
                 Mode A: op-node v1.19.7 + Engine API :9551 + JWT
                 Mode B: not year-1

 Interop         quub-gateway
                 CCTP V2 for USDC
                 source chain ≠ 8091

 Ops             F212 hash now; file store later
                 F211 freeze instant; unfreeze two-key
                 Travel-rule hash into F201 when a vendor exists
```

### What “no token” means

- Genesis does not mint a QUUB coin.  
- F213 takes listed F210 (dummy) or a listed stable later — not a protocol coin.  
- README: “Quub is a payments fabric. There is no native token.”  
- `rg` must not introduce a product ticker in contracts.

---

## View 4 — Deployment (what runs where)

Today the physical host is **one laptop**, loopback only. See `DEPLOYMENT.md` and `PHYSICAL.md`.

### Topology A — Engineer `--dev` (now)

```
operator-api :8080  ──►  quub-node --dev :8545
                         chain 8091
                         OwnerA=anvil0  OwnerB=anvil1
                         feeRecipient=0x…FEE0
```

Fake money. Anvil keys. Named datadir preferred (C035). Never production funds.

### Topology B — Mode A local (now)

```
operator-api :8080  ──►  quub-node --engine :9545
                              ▲
                              │ Engine API :9551 + JWT
                         op-node v1.19.7
                              │
                         geth 1.16.x :8546
                         full L1 genesis (not anvil)
                         SystemConfig 0xe991…F990
                         L2 genesis = OP alloc ∪ F210–F213
```

Kill `op-node` → L2 head must freeze. Mode A L2 **OwnerB** may still equal OwnerA until C034. Do not call that dual-control.

### Topology C — Gateway + rail (Sprint 6)

Same as A or B, plus:

```
quub-gateway  ──►  Ethereum Sepolia USDC   (burn, live)
              ──►  Circle attestation
              ──►  Base Sepolia USDC       (mint)
              ──►  mock://  if env missing
```

Quub 8091 never sees `depositForBurn`.

### Topology D — Public L1 (later)

Sepolia then Ethereum. `op-batcher` + `op-proposer`. Verifier host without sequencer JWT. TLS on `api.quub.network` / `rpc.quub.network`. KMS. Not standing.

### Topology E — Mode B sovereign (not year-1)

Named validators, Simplex, no Ethereum DA. Same EVM crate. Separate binary. Do not draw this as current.

### Environments

| Name | Chain id | Who | Money |
|---|---|---|---|
| `--dev` / Mode A local | **8091** | engineers | fake QPT |
| Sepolia L2 (later) | **8091** unless an ADR says otherwise | design partners | test USDC via CCTP |
| Production | **8091** until an ADR changes it | licensed client | real USDC on Base; Quub still identity/policy |

Do not invent chain id 8090.

### What must never sit in the `quub-node` image

- Sequencer / owner private keys (KMS later; anvil only on `--dev`)  
- Circle / operator Bearer secrets  
- Full ISO XML  
- Engine JWT on a public NIC  

---

## How to explain it in one breath

Quub sits **beside** Base and SWIFT — not as a wrapped dollar, and not as a coin you buy to pay.

- The **dollar** stays Circle’s dollar (on `--dev`, F210 is a dummy).  
- The **public rail** stays where the counterparty already is (Base first).  
- The **bank file** stays ISO-shaped; XML ingest is later.  
- Quub is where a licensed institution puts **rules, identity, evidence**, and year-1 **records the payment before the rail moves**.  
- `operator-api` is how ops talks to that without Solidity.  
- Nobody buys a Quub coin to use it.
