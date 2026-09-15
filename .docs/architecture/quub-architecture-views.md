# Quub — Architecture views
**Product:** Quub, a payments blockchain  
**Date:** 15 September 2026  
**Rule:** one product, no native token, no second brand

Four views. Same system. Different altitude.

1. **Ecosystem** — where Quub sits among rails that already exist  
2. **System** — the Quub parts and who talks to whom  
3. **Implementation** — how those parts are built  
4. **Deployment** — what runs where, in which phase  

---

## View 1 — Ecosystem (where Quub plays)

Quub does not replace SWIFT, Solana, Base, Tempo, Arc, or a bank core. It is a **payment settlement layer** that a licensed institution can contract, with a clean exit onto the rail the counterparty already uses.

```
                         VALUE MOVES HERE
  USDC / USDT / CAD-stable / AED-stable
           │
           │  native issuance (Circle, Tether, local issuer)
           ▼
 ┌─────────┴──────────┬─────────────┬──────────────┐
 │ Public payment     │ Quub        │ Bank / RTGS  │
 │ rails              │ settlement  │ messaging    │
 │                    │ layer       │              │
 │ Solana  USDC       │ Policy      │ SWIFT CBPR+  │
 │ Base    USDC       │ Memo        │ RTR (CA)     │
 │ Tempo   TIP-20     │ Paymaster   │ Aani (UAE)   │
 │ Arc     USDC       │ Lane        │ Fedwire etc. │
 │ Tron    USDT       │ Token       │ Lynx / UAFTS │
 └─────────┬──────────┴──────┬──────┴──────┬───────┘
           │                 │             │
           └────────────┬────┴─────────────┘
                        ▼
                 Quub Gateway
           (the only door a client uses)
                        ▲
                        │
              PSP / bank / EMI / agent
```

**What already exists and we do not rebuild**

| Layer | Who owns it | Quub’s relationship |
|---|---|---|
| Dollar inventory | Circle USDC, Tether USDT, local issuers | Use native paper. Never wrap as the unit of record |
| Public cheap rails | Solana, Base, Tempo, Arc, Tron | Counterparties live here. Gateway exits here via CCTP / official issuer paths |
| Bank messaging | SWIFT, RTR, Aani | Gateway emits/ingests ISO 20022. Quub does not run a SWIFT switch |
| Custody | Fireblocks, bank custody, Coinbase Prime | Standard `eth_*`. No custom adapter year-1 |
| Travel Rule | Notabene / equivalent | Vendor writes a hash. Quub Policy checks the hash |
| Identity / KYC | the licensed client | Quub does not become a KYC vendor |

**Where Quub is the system of record**

Only for the payment that needs *protocol* guarantees:

- freeze / allow / dual-control cannot be skipped by calling the token a different way  
- ISO identity is committed on the receipt  
- gas is a listed stable  
- mempool junk cannot consume the payment lane  
- evidence hash is on-chain; the file is in Quub Evidence  

If the payment does not need that, Gateway settles it on Base or Solana and Quub never holds the funds. That is still Quub. The chain is optional; the payment layer is not.

**What we charge for**

Orchestration and settlement service: Gateway fees, corridor FX, compliance pack, later sequencer fees in stables. Not a ticker.

---

## View 2 — System (components and contracts)

```
                         Client systems
              ERP · PSP core · treasury · agent
                         │
                         │  pain.001 / REST / x402
                         ▼
              ┌─────────────────────┐
              │    Quub Gateway     │
              │  API · route · CCTP │
              └──────────┬──────────┘
           ┌─────────────┼──────────────┐
           ▼             ▼              ▼
    Quub Policy    Quub ISO       Quub Evidence
    Engine         mapper         file store
           │             │              │
           └─────────────┼──────────────┘
                         ▼
              ┌─────────────────────┐
              │     Quub Node       │
              │   quub-node binary  │
              │   eth_* JSON-RPC    │
              └──────────┬──────────┘
                         │
         ┌───────────────┼────────────────┐
         ▼               ▼                ▼
   Execution         Consensus        Data
   Reth + EVM        Mode A OP        Mode A: Ethereum blobs
   F201 Policy       Mode B Simplex   Mode B: the chain itself
   F202 Memo
   F203 Paymaster
   F210 Token
   F211 Policy Admin
   F212 Anchor
   Payment lane
```

### On-chain (consensus-critical)

| Component | Address | Responsibility |
|---|---|---|
| Quub Policy | `F201` | Atomic allow / deny / hold. Fail closed |
| Quub Memo | `F202` | Hash of ISO identity set. No PII |
| Quub Paymaster | `F203` | Quote + debit gas in a listed stable |
| Quub Token | `F210` | The payment asset (USDC representation or local stable) |
| Quub Policy Admin | `F211` | Freezes, limits, fee-token list. Dual-control + timelock |
| Quub Anchor | `F212` | `packHash` of the off-block evidence file |
| Quub Fee Entry | `F213` | Solidity wrapper over F203 |
| Quub Lane | mempool + payload | 70% of block gas reserved for real payments |

### Node-adjacent (still Quub; cannot live in a public block)

| Component | Responsibility | Why off-block |
|---|---|---|
| Quub Gateway | Client API; route; CCTP / issuer exit | Talks to Circle, other RPCs, bank files |
| Quub Policy Engine | Same `check()` as F201, for pre-check and for public-rail hops | Must run before a tx is signed |
| Quub ISO | `pacs.008` / `pain.001` ↔ F202 fields | Full XML is PII |
| Quub Evidence | Store the file; give Gateway a locator | Supervisors read files, not calldata |

### External systems Quub talks to

- Circle CCTP V2 (native USDC burn/mint)  
- Public RPCs: Base, Solana, Tempo, Arc  
- Travel Rule vendor  
- Client IdP / dual-control keys (HSM / Fireblocks) — keys never in the Quub repo  

### What a payment looks like (sequence)

```
Client          Gateway        Policy/ISO/Evidence       Node (F201–F212)      Public rail
  │                │                    │                      │                    │
  │  pain.001      │                    │                      │                    │
  │───────────────►│                    │                      │                    │
  │                │  pre-check         │                      │                    │
  │                │───────────────────►│                      │                    │
  │                │  map ISO           │                      │                    │
  │                │  store file        │                      │                    │
  │                │  decide rail       │                      │                    │
  │                │                    │                      │                    │
  │                │──────── settle on Quub ──────────────────►│                    │
  │                │                    │              F201 allow                    │
  │                │                    │              F202 memo                     │
  │                │                    │              F203 gas                      │
  │                │                    │              F212 packHash                 │
  │                │                    │                      │                    │
  │                │──────── or exit USDC via CCTP ─────────────────────────────────►│
  │                │                    │                      │           native USDC
  │  pacs.008 / recon file              │                      │                    │
  │◄───────────────│                    │                      │                    │
```

Two legal outcomes, one product:

- **Settle on Quub** — Quub is the system of record for that payment.  
- **Settle on a public rail** — Quub Gateway + Policy + ISO still ran; funds never touched Quub Chain.  

---

## View 3 — Implementation (how it is built)

### Repository

```
quub/
  crates/
    quub-primitives      # addresses, reason codes, memo types
    quub-precompiles     # F201/F202/F203 bodies — NO reth dependency
    quub-evm             # QuubEvmFactory injects those precompiles into Reth
    quub-pool            # is_payment classifier
    quub-payload         # two-pass fill (later sprint)
    quub-node            # binary, feature flags mode-a | mode-b
    quub-consensus-op    # Mode A adapter (Sprint 2)
    quub-consensus-simplex
    quub-rpc             # eth_* only; quub_ off
  services/
    quub-gateway
    quub-policy
    quub-iso
    quub-evidence
  contracts/             # Foundry: Token, PolicyAdmin, Anchor, Fee Entry
  specs/
```

### Build rules

- Node language: Rust. Money contracts: Solidity + Foundry.  
- Reth **v2.5.2** as a library. Do not fork. Do not override `revm`.  
- Precompile crate has zero Reth deps so Mode A and Mode B share one execution crate.  
- Policy Engine and F201 call the same `check()` function. One spec, two hosts.  
- JSON-RPC is standard `eth_*` so Fireblocks / MetaMask / Alloy / Foundry work unchanged.  
- No native token. `--dev` ETH is a miner convenience, not the product.  
- No new transaction type in year 1. Classify existing txs for the payment lane.

### Stack by layer

```
 Application          Foundry contracts + Gateway API (HTTP / ISO file)
                      Alloy / viem on the client side

 Execution            quub-evm  →  Reth 2.5  →  REVM
                      PrecompilesMap + F201 F202 F203

 Settlement asset     F210 token / native USDC on public rails
                      F203 fee in listed stable

 Consensus            Mode A: op-node Engine API, Ethereum blobs
                      Mode B: Commonware Simplex, named validators
                      Feature flags. Never both in one binary.

 Interop              Quub Gateway
                      CCTP V2 for USDC
                      official issuer paths only
                      ISO 20022 in/out

 Ops                  Evidence store + packHash on F212
                      dual-control on F211
                      Travel Rule hash into F201
```

### What “no token” means in the implementation

- Genesis does not mint a QUUB coin.  
- Block rewards, if any under Mode B, are paid in the listed fee stable or are zero (permissioned set).  
- F203 `quote` / `takeFee` take USDC (or CADD / AED-stable), not a protocol coin.  
- README and Gateway copy: “Quub is a payments fabric. There is no native token.”

---

## View 4 — Deployment (what runs where)

Three topologies. Same software. Different blast radius.

### Topology A — Gateway only (now → first clients)

No public Quub chain. Gateway + Policy + ISO + Evidence talk to **existing** RPCs.

```
           [ client VPC / Quub VPC ]
 Quub Gateway ──► Base RPC
              ──► Solana RPC
              ──► Tempo RPC
              ──► Circle CCTP API
              ──► Notabene
              ──► Quub Evidence (object store + DB)
 quub-node --dev                 # engineers only, not production money
```

Use when: Phase 0–1, proving the book. Production money never sits on Quub Chain.

### Topology B — Mode A rollup (first Quub zone)

```
 Licensed client ──► Quub Gateway (active-active, two regions)

 Quub Gateway ──► quub-node (sequencer, HSM / Fireblocks key)
              ──► quub-node (replica RPC)
              ──► op-node   (derivation)
              ──► Ethereum L1 + blob DA
              ──► Circle CCTP (exit to Base / Solana / Ethereum)
              ──► Quub Evidence
              ──► Quub ISO (file in / file out to SWIFT service bureau)

 Sequencer is a licensed entity. Not an anonymous set.
```

Use when: two design partners asked for a dedicated zone. Economic security is Ethereum. Exit is CCTP via L1 or via a listed domain once Circle adds Quub.

### Topology C — Mode B sovereign (only if named FIs fund it)

```
 Validator 1 (Bank A)  ─┐
 Validator 2 (EMI B)   ─┼─ Commonware Simplex ── quub-node (same evm crate)
 Validator 3 (PSP C)   ─┘
 Quub Gateway in front, same as B.
 No Ethereum DA. Finality ~0.5s. You own the security budget.
```

Use when: a consortium will run 2f+1 validators and accept that risk. Same Gateway, same F201–F203. Different consensus feature flag.

### Environments

| Name | Chain id | Who uses it | Money |
|---|---|---|---|
| `quub-local` | 8091 | engineers, `quub-node --dev` | fake |
| `quub-sepolia` | TBD | design partners | test USDC |
| `quub` mainnet | 8090 placeholder | production | real stables only after Phase 1 gate |

Confirm chain ids on chainid.network before freeze.

### What must never be in the same image as `quub-node`

- Sequencer / validator private keys (HSM / MPC)  
- Notabene / Circle / Fireblocks API secrets  
- Full ISO XML  
- Production chain spec until counsel and ops sign off  

Gateway, Evidence, and the node scale independently. The node is stateful. Gateway is stateless-enough to run 2+. Evidence is an object store + DB.

---

## How to explain it in one breath

Quub is a payment layer that sits **beside** Solana, Base, Tempo, and SWIFT — not on top of them as a wrap, and not underneath them as a new settlement asset.

- The **dollar** stays the issuer’s dollar.  
- The **public rail** stays where the counterparty already is.  
- The **bank file** stays ISO 20022.  
- Quub is the place a licensed institution puts the *rules*, the *identity*, and optionally the *settlement* when public rails will not carry them.  
- Quub Gateway is how that institution talks to all of the above without learning a second product name.  
- Nobody buys a Quub coin to use it.
