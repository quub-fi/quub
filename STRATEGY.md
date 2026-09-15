# Quub — First Assessment

**Network:** Quub  
**Company:** fazeZERO (legal entity only; customers and code hear Quub)  
**Date:** 15 September 2026  
**Audience:** founders, design partners, engineers  
**Purpose:** the market decision, the white space, the entry, and what building Quub actually means.

There is one product. Quub is the payments blockchain. Gateway, ISO, Evidence, and Policy Engine are Quub components, not a second brand.

---

## Decision in one page

Do not start by building Solana. Do not start on Substrate / Polkadot. Do not launch a token.

Build a payments chain whose protocol does what a licensed payment must do:

- **Quub Policy (F201)** — allow / deny / freeze / Travel Rule / dual-control before a transfer commits
- **Quub Memo (F202)** — ISO payment identity on the receipt (EndToEndId, UETR, InstrId, CCY)
- **Quub Paymaster (F203)** — gas in a listed stable, not a chain coin
- **Quub Lane** — reserved blockspace so junk cannot eat payroll
- **Quub Gateway / ISO / Evidence** — how a client and a bank file talk to that chain (PII never goes on-chain)

A client talks to Quub. If a dollar must exit as native USDC on Base or Solana, **Quub Gateway** uses CCTP. Same payment, same UETR.

**Stack**

- Language of the node: Rust
- Language of public contracts: Solidity (Foundry)
- Execution: Reth SDK + REVM, EVM-equivalent
- Default consensus (Mode A): OP Stack rollup, `op-reth` + `op-node`, Ethereum blob DA
- Optional consensus (Mode B): Commonware Simplex BFT, permissioned named-FI validator set, only if a consortium funds it
- Not Substrate. Not a new VM. Not a volatile gas token. Not a token-first launch.

This is how Tempo shipped (Reth + Simplex), how Arc shipped (Reth + Malachite), and how Base’s node is built (Reth library, not a Reth fork).

Sites: `quub.network` = protocol. `quub.fi` = Gateway / operator UI for the same chain.

---

## 1. What the market actually looks like (September 2026)

The general-purpose L1/L2 market is crowded. The regulated settlement layer is not.

Stablecoin supply on tracked networks is about **$307B**. Ethereum ~48%, Tron ~31%, BNB Chain ~5.7%, Solana ~5.3%. Everything else, including Base, Arbitrum, Plasma, XRPL and Stellar, fights over the rest.

Activity is not in the same place as supply:

- **Ethereum** — reserve and issuer settlement. Not a retail payment rail.
- **Tron** — informal dollar rail. ~93% of stablecoin transfer volume is P2P. The moat is USDT-TRC20, not tech.
- **Solana** — high-velocity consumer and payout rail. Sub-cent fees, native USDC, Visa / PayPal / x402. Median tickets on payment flows are often under $100.
- **Base** — distribution L2. Coinbase users, zero-fee USDC withdrawals onto Base. OP Stack is commodity. The customer book is not.
- **BNB Chain** — Binance’s attached chain. Same law as Base: exchange first, ledger second.

A new category — **stablechains** — already exists and is owned by the three parties who control the money:

| Chain | Backer | Status (Sep 2026) | What they brought |
| --- | --- | --- | --- |
| Tempo | Stripe + Paradigm | Mainnet since 18 Mar 2026 | Merchant + payout book, stablecoin gas, ISO memos, policy registry, Machine Payments Protocol |
| Arc | Circle | Public mainnet 16 Sep 2026 | USDC issuance, OCC national trust, validators including BlackRock, DTCC, Visa, Mastercard |
| Plasma / Stable | Tether orbit | Live | USDT-as-gas, remittance / offshore dollar flows |

Tempo raised at a $5B valuation. Plasma ran a $373M token sale. Arc’s genesis validator set is the most institutionally concentrated public L1 launch on record.

Generalist new L1s and L2s are being shut off, not scaled. Harmony proposed sunsetting its chain. BounceBit left its L1. Throughput is no longer a differentiator. Fees on Solana, Base, Tempo and Tron are already low enough for payments.

The scarce assets are **distribution, licenses, corridor liquidity, and an operating model a supervisor will accept.**

---

## 2. What the winners actually won

Every surviving payments chain won a captive flow, then attached a ledger.

- Coinbase had the users, then launched Base.
- Stripe had the merchants, then launched Tempo.
- Circle had USDC + bank relationships, then launched Arc.
- Binance had the exchange, then grew BNB Chain.
- Tether had the offshore dollar; Tron (and now Plasma/Stable) carried it.

Solana is the exception that proves the rule: a decade of consumer UX, then acquirers landed on it. Nobody repeats that from a whitepaper in 2026.

A boutique firm cannot win a $100M–$500M distribution war against Stripe, Circle, Tether, Coinbase and Binance. It can win a mid-market book those platforms serve badly — if Quub is the chain that book settles on.

---

## 3. Five white spaces (ranked)

### 1. Regulated settlement fabric between banks and public chains

Highest fit. Tempo / Arc / Base / Solana optimize public stablecoin movement. Licensed PSPs and banks still need dual-control, ISO 20022 envelopes, Travel Rule, freeze/allow policy, and an evidence pack a supervisor can read — plus a clean exit onto native USDC/USDT.

That pack is Quub Policy + Quub Memo + Quub Evidence + Quub Gateway. Canton occupies the capital-markets version. SWIFT’s ledger is live with tokenized deposits. Rayls is doing bank-sovereign ledgers in Brazil. Canada–US–GCC mid-market is underbuilt.

### 2. Long-tail corridor FX and local-currency stables

Highest commercial gap. Dollar stables are a solved rail. Non-USD stables grew from ~$44M (2023) to ~$2.2B (2026) against $300B+ of USD paper. On-chain FX is still Base + euro. Working corridors: US–Mexico, US–Philippines, Argentina, Nigeria, UAE–Pakistan. SSA remittance corridors still clear above 20% all-in.

Do **not** issue a USD stablecoin. GENIUS Act turns that into a licensed PPSI business. Partner with USDC / PYUSD / a CAD- or AED-regulated issuer. Own FX, off-ramp, and policy. First corridor: CAD–USD or UAE–South Asia.

### 3. Accountability layer for agentic payments

x402 and Tempo MPP solved “pay per request.” Banks will ask: who paid, on whose authority, under what policy, with what audit trail. That is Quub Policy + Quub Evidence on top of those rails. Not an “agent L1.”

### 4. Issuer-neutral multilateral oracle + FX RFQ

Chainlink has the bank consortium version. Arc has a native FX engine. The gap is mid-market CAD / AED / SAR / BRL / NGN RFQ with bank-grade attestations, not locked to Circle. Build as a Quub Gateway module. A specialized appchain only if volume justifies it.

### 5. Distribution-first L2 attached to a real customer book

The only version of “our own network” that has worked since 2023. It only works if the book exists first. Token last, if ever. Base still has no token and is the L2 that makes money.

---

## 4. Architecture decision

| Path | Year-1 cash | Team | Time to useful product | Verdict |
| --- | --- | --- | --- | --- |
| Custom general-purpose L1 | $50–200M+ | 40–80 | 36–60 months | Do not do |
| Stablechain L1 clone | $30–80M | 25–50 | 24–36 months | Do not start here |
| Substrate / Polkadot SDK L1 | $20M+ and the wrong market | FRAME specialists | 24+ months | Reject. Rust ≠ Substrate |
| Serious payments L2 / appchain | $5–15M | 15–30 | 12–18 months | Phase 2, conditional |
| Quub Gateway + Policy + ISO on public rails, then Quub Node | $2–8M then $5–15M | 8–15 then 15–30 | 3–9 months to first client; 12–18 to a zone | **Start here** |

Substrate is excellent engineering and the wrong gravity well. Polkadot carries ~$69M of stables (rank ~24). Circle, Fireblocks, Visa and bank counsel live on EVM. Tempo’s team maintains Reth and still chose EVM semantics. That is the tell.

**Future-proof stack:** Rust node, EVM state, swappable consensus, RISC-V zk proofs attachable later (SP1 / RISC0) without changing the VM issuers deployed to.

---

## 5. Thirty-six month plan

### Phase 0 — now to month 6

Do not announce a public mainnet.

- Run **Quub Gateway + Quub Policy + Quub ISO + Quub Evidence** against Solana, Base, Tempo, Arc. Same `check()` and same ISO identity set that will later sit at F201 / F202.
- One corridor, boringly excellent: CAD–USD or UAE–South Asia.
- No token. No foundation. No testnet incentive program.
- Gate: 3 licensed production clients, first-transfer to recurring payout, a one-page model for the first 10 million transactions.

This is Quub software using public rails until `quub-node` is ready. It is not a second company.

### Phase 1 — months 6 to 18

Become the operating layer those clients cannot rip out.

- 8–15 live regulated clients. Target $500M–$2B annualized volume through Quub Gateway.
- Policy engine portable across public rails (application-layer version of Tempo TIP-403) and identical to F201.
- Raise $3–8M only with named design partners for a dedicated Quub zone: one bank or EMI, one PSP, one corridor LP.
- Gate: at least two clients asking for that zone. If nobody asks, do not start Quub mainnet.

### Phase 2 — months 18 to 36

Launch Quub mainnet as the settlement zone those clients already use.

- Default: Mode A, OP Stack rollup, stablecoin gas, F201–F203, CCTP exit via Quub Gateway.
- Genesis traffic is the existing book. Design partners run the sequencer.
- Year-1 chain budget: $5–15M all-in.
- Apps on top: treasury sweeps, payroll, merchant payout, agent wallets with spend policy.
- Metric: $5B+ annualized through the fabric, 1–2 exclusive anchors, fee revenue that does not depend on a token.

### Phase 3 — years 3 to 5

Only now is “compete with established networks” a serious sentence — and only as specialized blockspace for mid-market regulated flow, long-tail corridors, and ISO-native settlement. Interop is the growth strategy. Isolation is how new L1s die.

---

## 6. Kill criteria

- Cannot sign 3 licensed production clients in 12 months → do not start public mainnet.
- Cannot explain the captive flow that fills the first 10 million transactions → do not start public mainnet.
- Token launch becomes the plan → abort.
- Strategy depends on out-TPS Solana or out-USDT Tron → abort.
- No bank, EMI or licensed PSP will run a sequencer / validator → keep Gateway on public rails.
- Year-1 cash requires a speculative token → architecture is wrong.

---

## 7. Name and domains

**Quub** is the network name. Always spell Q-U-U-B. Never say Qubic.

**Owned:** `quub.fi`, `quub.network`. Ownership of a domain is not a trademark.

| Host | Role |
| --- | --- |
| `quub.network` | Protocol, docs, later RPC and explorer |
| `quub.fi` | Quub Gateway / operator UI for the same chain |

Park both on a host you control before sending either URL to a bank. Collisions that remain: Qubic (live L1, spoken near-twin) and Quub Inc. / `quub.space` (Lancaster satellite company). Counsel still runs CA / UAE / US class 36 and 42. Never use tickers QUB, QUBE, QUBC or QUBIC. There is no native token.

---

## 8. What to hand whom

| Audience | Artifact |
| --- | --- |
| Founders, partners, design-partner banks | This assessment |
| Engineers (human + Cursor) | `AGENTS.md`, `.cursorrules`, `SPRINT.md`, NodeBuilder sketch |
| Counsel | Name collisions + vendor pack in the NodeBuilder sketch |

The next engineering move is `quub-node --dev` with F201–F203 live, and every first-party file saying Quub only.
