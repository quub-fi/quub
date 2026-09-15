# Quub — First Assessment

**Working name of the network:** Quub
**Company / orchestration product:** fazeZERO / xZERO (unchanged)
**Date:** 15 September 2026
**Audience:** founders, design partners, engineers
**Purpose:** the market decision, the white space, the entry, and what “build our own network” actually means. Architecture details live in the NodeBuilder pack and the Cursor prompt.

---

## Decision in one page

Do not start by building Solana. Do not start on Substrate / Polkadot.

Start by owning a regulated payment book on rails that already work (Solana, Base, Tempo, Arc, Tron). Graduate that book onto a purpose-built network whose validators or sequencer are the same licensed institutions already sending the volume.

That network is **Quub**.

- Quub is the ledger.
- xZERO is the off-chain orchestration, ISO 20022 mapper, dual-control, and evidence plane.
- fazeZERO is the firm.

The chain is an implementation detail of a compliance-native payment OS. Without a captive flow, Quub is a museum.

**Stack, if and when the ledger is justified**

- Language of the node: Rust
- Language of public contracts: Solidity (Foundry)
- Execution: Reth SDK + REVM, EVM-equivalent
- Default consensus (Mode A): OP Stack rollup, `op-reth` + `op-node`, Ethereum blob DA
- Optional consensus (Mode B): Commonware Simplex BFT, permissioned named-FI validator set, only if a consortium funds it
- Not Substrate. Not a new VM. Not a volatile gas token. Not a token-first launch.

This is how Tempo shipped (Reth + Simplex), how Arc shipped (Reth + Malachite), and how Base’s node is built (Reth library, not a Reth fork).

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

| Chain           | Backer            | Status (Sep 2026)          | What they brought                                                                             |
| --------------- | ----------------- | -------------------------- | --------------------------------------------------------------------------------------------- |
| Tempo           | Stripe + Paradigm | Mainnet since 18 Mar 2026  | Merchant + payout book, stablecoin gas, ISO memos, policy registry, Machine Payments Protocol |
| Arc             | Circle            | Public mainnet 16 Sep 2026 | USDC issuance, OCC national trust, validators including BlackRock, DTCC, Visa, Mastercard     |
| Plasma / Stable | Tether orbit      | Live                       | USDT-as-gas, remittance / offshore dollar flows                                               |

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

A boutique regulated-payments firm cannot win a $100M–$500M distribution war against Stripe, Circle, Tether, Coinbase and Binance. It can win a mid-market book those platforms serve badly.

---

## 3. Five white spaces (ranked for this firm)

### 1. Regulated settlement fabric between banks and public chains

Highest fit. Tempo / Arc / Base / Solana optimize public stablecoin movement. Licensed PSPs and banks still need dual-control, ISO 20022 envelopes, Travel Rule, freeze/allow policy, and an evidence plane a supervisor can read — plus a clean exit onto USDC/USDT on public rails.

Canton occupies the capital-markets version. SWIFT’s ledger is live with tokenized deposits. Rayls is doing bank-sovereign ledgers in Brazil. Canada–US–GCC mid-market is underbuilt. That is the current xZERO book.

### 2. Long-tail corridor FX and local-currency stables

Highest commercial gap. Dollar stables are a solved rail. Non-USD stables grew from ~$44M (2023) to ~$2.2B (2026) against $300B+ of USD paper. On-chain FX is still Base + euro. Working corridors: US–Mexico, US–Philippines, Argentina, Nigeria, UAE–Pakistan. SSA remittance corridors still clear above 20% all-in.

Do **not** issue a USD stablecoin. GENIUS Act turns that into a licensed PPSI business. Partner with USDC / PYUSD / a CAD- or AED-regulated issuer. Own FX, off-ramp, and compliance orchestration. First corridor: CAD–USD or UAE–South Asia.

### 3. Accountability layer for agentic payments

x402 and Tempo MPP solved “pay per request.” Banks will ask: who paid, on whose authority, under what policy, with what audit trail. That is identity + policy + evidence on top of those rails. Not an “agent L1.”

### 4. Issuer-neutral multilateral oracle + FX RFQ

Chainlink has the bank consortium version. Arc has a native FX engine. The gap is mid-market CAD / AED / SAR / BRL / NGN RFQ with bank-grade attestations, not locked to Circle. Build as a product. A specialized appchain only if volume justifies it.

### 5. Distribution-first L2 attached to a real customer book

The only version of “our own network” that has worked since 2023. It only works if the book exists first. Token last, if ever. Base still has no token and is the L2 that makes money.

---

## 4. Architecture decision

| Path                             | Year-1 cash                | Team              | Time to useful product | Verdict                   |
| -------------------------------- | -------------------------- | ----------------- | ---------------------- | ------------------------- |
| Custom general-purpose L1        | $50–200M+                  | 40–80             | 36–60 months           | Do not do                 |
| Stablechain L1 clone             | $30–80M                    | 25–50             | 24–36 months           | Do not start here         |
| Substrate / Polkadot SDK L1      | $20M+ and the wrong market | FRAME specialists | 24+ months             | Reject. Rust ≠ Substrate  |
| Serious payments L2 / appchain   | $5–15M                     | 15–30             | 12–18 months           | Phase 2, conditional      |
| Multi-rail orchestration (xZERO) | $2–8M                      | 8–15              | 3–9 months             | **Phase 0–1. Start here** |

Substrate is excellent engineering and the wrong gravity well. Polkadot carries ~$69M of stables (rank ~24). Circle, Fireblocks, Visa and bank counsel live on EVM. Tempo’s team maintains Reth and still chose EVM semantics. That is the tell.

**Future-proof stack:** Rust node, EVM state, swappable consensus, RISC-V zk proofs attachable later (SP1 / RISC0) without changing the VM issuers deployed to.

---

## 5. Thirty-six month plan

### Phase 0 — now to month 6

Do not announce a chain.

- Productize xZERO as multi-rail orchestration: Solana + Base + Tron + Tempo + Arc.
- ISO 20022 in/out, dual-control, Travel Rule, evidence plane.
- One corridor, boringly excellent: CAD–USD or UAE–South Asia.
- No token. No foundation. No testnet incentive program.
- Gate: 3 licensed production clients, first-transfer to recurring payout, a one-page captive-flow model for the first 10 million transactions.

### Phase 1 — months 6 to 18

Become the operating layer those clients cannot rip out.

- 8–15 live regulated clients. Target $500M–$2B annualized orchestrated volume.
- Policy engine portable across public rails (the application-layer version of Tempo TIP-403).
- Raise $3–8M only with named design partners for a private settlement zone: one bank or EMI, one PSP, one corridor LP.
- Gate: at least two clients asking for a dedicated settlement zone. If nobody asks, do not start Quub mainnet.

### Phase 2 — months 18 to 36

Launch Quub as an implementation detail of the product.

- Default: Mode A, OP Stack rollup, stablecoin gas, compliance precompiles, CCTP / Bridge exit.
- Genesis traffic is the existing book. Design partners run the sequencer.
- Year-1 chain budget: $5–15M all-in.
- Apps on top: treasury sweeps, payroll, merchant payout, agent wallets with spend policy.
- Metric: $5B+ annualized through the fabric, 1–2 exclusive anchors, fee revenue that does not depend on a token.

### Phase 3 — years 3 to 5

Only now is “compete with established networks” a serious sentence — and only as specialized blockspace for mid-market regulated flow, long-tail corridors, and ISO-native settlement. Interop is the growth strategy. Isolation is how new L1s die.

---

## 6. Kill criteria

- Cannot sign 3 licensed production clients in 12 months → do not start the chain.
- Cannot explain the captive flow that fills the first 10 million transactions → do not start the chain.
- Token launch becomes the plan → abort.
- Strategy depends on out-TPS Solana or out-USDT Tron → abort.
- No bank, EMI or licensed PSP will run a sequencer / validator → stay on public rails.
- Year-1 cash exceeds services + a modest raise without a speculative token → architecture is wrong.

---

## 7. Name

**Quub** is a working network name. Use it in code and internal docs.

Do not print public letterhead until counsel runs CA / UAE / US class 36 and 42 searches. Collisions: Qubic (live L1, spoken near-twin), quub.fi (DeFi site), Quub Inc. / quub.space (Lancaster satellite company, already pronounced “cube”). Never use tickers QUB, QUBE, QUBC or QUBIC. There is no native token.

---

## 8. What to hand whom

| Audience                                 | Artifact                                                        |
| ---------------------------------------- | --------------------------------------------------------------- |
| Founders, partners, design-partner banks | This assessment                                                 |
| Engineers (human + Cursor)               | `quub-nodebuilder-sketch.md` + `quub-cursor-prompt.md`          |
| Counsel                                  | Name collisions + vendor pack section in the NodeBuilder sketch |

The next engineering move is not a whitepaper. It is a Reth workspace named `quub-node` with three precompile stubs and a Mode A/B feature flag, while xZERO continues to run against public rails.
