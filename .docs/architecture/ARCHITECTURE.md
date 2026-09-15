# Quub — Locked architecture
**Date:** 15 September 2026  
**Status:** binding for engineers, Cursor, and design partners  
**Network:** Quub  
**Company:** fazeZERO (legal entity only; customers and code hear Quub)

There is one product. Quub is a payments blockchain. Gateway, ISO, Evidence, Policy, and the operator API are Quub components, not a second brand. There is no native token.

Sites: `quub.network` = protocol. `quub.fi` = operator UI.

---

## 0. What Quub is

A payment **policy and identity** layer that a licensed institution can contract. Year-1 it also **records** the payment on chain 8091 before a dollar exits via Circle CCTP.

It does **not** replace Ethereum, Solana, Base, SWIFT, or a bank core.  
It does **not** issue the dollar.  
It does **not** require anyone to buy a chain coin.

It does:

- decide whether a payment may move (F201 / F211)
- commit ISO payment identity on the receipt (F202)
- take a listed-token fee on identified memos only (F213)
- reserve a lane so junk cannot eat payroll (70/30)
- anchor an evidence hash (F212)
- exit **native USDC** onto Base via CCTP after that memo mines

`--dev` F210 is QPT dummy. It is not USDC and not a product ticker.

A future “gateway-only, 8091 not in the path” topology needs a new ADR. It is not year-1.

---

## 1. Architecture decisions (locked)

| ID | Decision | Why | Rejected |
|---|---|---|---|
| **ADR-1** | One product: Quub | Two brands confused the team and the client | xZERO as a sister product |
| **ADR-2** | No native / gas token | Banks will not buy a thin float to move a dollar | QUUB / QUB / QUBE / QUBIC ticker |
| **ADR-3** | Rust node, EVM execution, Solidity money contracts | Circle, Fireblocks, Visa, bank counsel, Foundry, Alloy | Substrate/FRAME, Cosmos SDK, SVM, Move, custom VM |
| **ADR-4** | Reth as a **library** (live pin = ADR-16) | Do not maintain a Quub fork of Reth | Forking Reth, floating `main`, overriding `revm` |
| **ADR-5** | Default consensus = **Mode A** OP Stack | Ethereum DA. Sequencer is a licensed operator | Public token-weighted L1; Tendermint; Substrate consensus |
| **ADR-6** | Optional consensus = **Mode B** Simplex | Only if named FIs fund 2f+1. Same `quub-evm` | Shipping both modes in one binary; Mode B as year-1 |
| **ADR-7** | Two launch paths: `--dev` miner and `--engine` + `op-node` | `--dev` for engineers. Mode A is the derived L2 | Anvil as Mode A L1; treating `--dev` as Mode A |
| **ADR-8** | Precompiles for policy / memo | Protocol-cost gates | Application-only policy |
| **ADR-9** | Frozen addresses F201–F203, F210–F213 | Stable for Foundry and the API | Tempo-style prefixes |
| **ADR-10** | No PII on-chain | Supervisors read files; chain stores hashes | Full `pacs.008` XML in calldata |
| **ADR-11** | JSON-RPC is `eth_*` | Fireblocks / MetaMask / Alloy / Foundry | `quub_` namespace; new EIP-2718 type in year 1 |
| **ADR-12** | Multi-chain by **adapters + official transports** | Circle owns USDC teleport | Quub-canonical wrapped dollar; “all chains” lock-and-mint |
| **ADR-13** | USDC hops = **CCTP V2**. Other tokens later = **CCIP** | Do not rebuild Circle or Chainlink | CCIP for USDC when CCTP exists; oracles inside F201 |
| **ADR-14** | Year-1 identified payment **records on 8091 first**, then rail | Freeze must bind before the dollar mints on Base | Skipping F201 on off-us; burn id = Quub tx |
| **ADR-15** | Spell Q-U-U-B. Never Qubic | Live L1 collision | Tickers QUB, QUBE, QUBC, QUBIC |
| **ADR-16** | Execution pin: `op-rs/reth` @ `aef8d3ef…` + `op-reth/v2.4.4` + `op-node` **v1.19.7**; rustc **1.96**; one Reth remote | Unlocks Mode A | `paradigmxyz/reth` v2.5.2; two Reth remotes; hand-rolled `rollup.json` |
| **ADR-17** | Payment lane: F210 + exact `transferWithMemo`; `fill_lanes` 70/30 + spill; lane beats tip; same fill on `--dev` and `--engine` | Reserve payment blockspace without a new tx type | Tip-only packing; 70/30 in PolicyAdmin; plain `transfer` as payment |
| **ADR-19** | Fee on memo only via F213 `takeFee` → `paymasterDebit`; freeze instant; unfreeze/params = other owner; `--dev` OwnerB = anvil1 | Fee moves + maker-checker without 4337 / 24h lock | Instant unfreeze; EOA `takeFee`; fee on plain `transfer`; `tx.gasprice` quote |
| **ADR-20** | First public rail = CCTP V2 → Base; Quub records only; mock valid offline; live ≤ 1 USDC | Official USDC hop | Quub as a CCTP domain; Solana/CCIP this year-1 sprint |

Full text: `docs/adr/ADR-016-execution-pin.md`, `ADR-017-payment-lane.md`, `ADR-019-paymaster-dual-control.md`, `ADR-020-cctp-base-rail.md`.

---

## 2. System context

```
 Originating systems          Public rails              Bank rails
 ERP · PSP · operator-api     Base USDC (year-1)        core / ISO files
         │                    Ethereum CCTP source      SWIFT later
         │                    Solana / CCIP later
         ▼
  operator-api :8080
  quub-iso · quub-gateway
         │
         ▼
    quub-node  (--dev :8545 or --engine :9545)
    F201 F202 F203 F210–F213 + lane
         │
    Mode A: op-node v1.19.7 + geth L1 (local)
            public L1 later
    Mode B: not year-1
```

Actors: licensed PSP / EMI, bank treasury, ops, later a supervisor.  
Quub does not custody client keys in production (KMS later). Quub does not run a SWIFT switch.

---

## 3. Containers

### On-chain

| Component | Address | Role | Status |
|---|---|---|---|
| Quub Policy | `0x…F201` | Stateful check. Caller = F210. Sloads F211. Fail closed | Shipped |
| Quub Memo | `0x…F202` | ISO identity hash + `tx.origin`. No PII | Shipped |
| Quub Paymaster precompile | `0x…F203` | Thin. Does not move funds | Stub shipped |
| Quub Payment Token | `0x…F210` | Asset on 8091. `--dev` = QPT dummy | Shipped |
| Quub Policy Admin | `0x…F211` | Freeze instant. Unfreeze / params two-key | Shipped |
| Quub Evidence Anchor | `0x…F212` | `packHash` + memoHash | Shipped |
| Quub Fee Entry | `0x…F213` | `quote` + `takeFee`. Only F210 may call | Shipped |
| Quub Lane | pool + payload | 70% gas for exact memos | Shipped |

Chain id: **8091** on `--dev` and Mode A. No 8090 until a new ADR.

### Node-adjacent

| Component | Role | Status |
|---|---|---|
| operator-api | HTTP: pay, status, freeze. `127.0.0.1:8080` | Shipped |
| quub-gateway | CCTP + `RailRecord` | Sprint 6 |
| quub-iso | Field checks (e2e, UETR, ccy) | Stub shipped |
| quub-policy | Reason codes | Shipped |
| Evidence file store | Bytes for `packHash` | Later |

```
quub.network/
  crates/   quub-primitives quub-precompiles quub-evm quub-pool
            quub-payload quub-node quub-consensus-op quub-consensus-simplex quub-rpc
  apps/     operator-api
  services/ quub-policy quub-iso quub-evidence quub-gateway
  contracts/
  specs/
  docs/adr/
```

Touched files must not contain `xzero` / `xZERO`.

---

## 4. Runtime paths

**A — identified on-us (shipped)**  
Client → operator-api → `transferWithMemo` on F210 → F201 → F213 fee → principal → F202 → F212.

**B — identified off-us (Sprint 6)**  
Same as A. Then `quub-gateway` CCTP burn on **Ethereum Sepolia** (or mock) and mint on **Base Sepolia**. `RailRecord[endToEndId]` holds both worlds. Burn id ≠ Quub tx.

**C — reject**  
F201 ≠ 0 → no fee, no principal, no burn.

**D — public-rail-only (not year-1)**  
Gateway talks to Base without a 8091 memo. Requires a new ADR.

**E — fiat last mile (later)**  
ISO file out to SWIFT / a bureau. The chain never speaks SWIFT.

---

## 5. Consensus

### Mode A — default (shipped locally)

- `--engine` HTTP :9545, Engine API :9551 + JWT  
- `op-node` v1.19.7 derives L2 from L1  
- Local L1 = **geth** with full genesis (SystemConfig `0xe991…F990`). Not anvil  
- Kill `op-node` → L2 head freezes  
- Public batcher / proposer / Sepolia L1 = later  

### `--dev`

- Same `QuubEvmFactory` + lane + contracts  
- Local miner :8545  
- Not Mode A. Two datadirs if both run  

### Mode B — not year-1

Simplex stub. Feature off. Same evm crate if a consortium later funds validators.

---

## 6. Execution and precompiles

- Pin per **ADR-16**. Alloy on Reth crates follows that pin. Services may stay on workspace Alloy 0.8 until a dedicated bump.  
- Inject F201–F203 when `spec >= PRAGUE` (or OP equivalent).  
- F201 and F202: `DynPrecompile::new_stateful`.  
- F201: `sload` F211 → policy check. Empty F211 → deny. Caller must be F210.  
- Slots from `forge inspect`. Typical PolicyAdmin: slot 0 `ownerA`; slot 1 `ownerB` + `paused`; slot 2 `threshold`; slot 3 `feeToken`; slot 4 `frozen` base.

### F202 hash (locked)

```
memoHash = keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, tx.origin))
```

No timestamp. No PII. Same formula in `quub-iso`, F202 wrap (`tx_origin()`, not `caller`), PaymentToken, Foundry.

A fixture that hashes origin `0x…00AA` will not match a mined `--dev` payment from anvil0 `0xf39F…`. Both are correct.

---

## 7. Interop

| Job | Owner | Quub |
|---|---|---|
| Native USDC hop | Circle CCTP V2 | Gateway after memo |
| Other tokens + messages | Chainlink CCIP | Later adapter |
| FX / NAV | Chainlink Data Streams | Gateway later. Never inside F201 |
| Who may pay, identity, evidence, lane | Quub | The product |

Year-1 adapter: **CCTP → Base**.  
Add a chain only when a licensed client has volume **and** an official asset path exists.

---

## 8. Deployment

| Topology | What runs | Money |
|---|---|---|
| `--dev` | `quub-node` :8545 + operator-api :8080 | Fake QPT. Anvil keys |
| Mode A local | geth :8546 + `--engine` :9545/:9551 + `op-node` | Fake. OwnerB may still = OwnerA on L2 |
| Rail | + `quub-gateway` mock or live ≤ 1 USDC | Test USDC only if env set |
| Public L1 | later | later |
| Mode B | not year-1 | — |

`--dev` OwnerA = anvil0, OwnerB = anvil1, feeRecipient = `0x…FEE0`.  
Never in the `quub-node` image: sequencer keys, vendor secrets, full ISO XML, Engine JWT on a public NIC.

See `DEPLOYMENT.md` and `PHYSICAL.md`.

---

## 9. Build sequence

| Sprint | Outcome | Status |
|---|---|---|
| **0** | Primitives, ISO/policy libs, Foundry facades | Done |
| **1** | `--dev`, F201–F203 registered | Done |
| **1.5** | Genesis F210–F213, stateful F201, live memo + freeze | Done |
| **2** | Mode A + ADR-16 pin | Done |
| **3** | Payment lane 70/30 | Done |
| **4** | operator-api :8080 | Done |
| **5** | F213 takeFee + two-key unfreeze | Done |
| **6** | CCTP → Base rail (mock valid) | In flight |
| **7** | API two-key unfreeze, Mode A OwnerB, named datadir, `rail` field | Next engineering |
| **Mode B** | Not a sprint until a consortium funds it | Out of year-1 |

Kill criteria: no 3 licensed clients in 12 months → no public mainnet. Token launch becomes the plan → abort. Out-TPS Solana → abort. Second Reth remote / Simplex in Mode A binary / wrapped USDC → revert.

---

## 10. What we will not build

- A Quub coin, stake, or airdrop  
- A canonical wrapped USDC on Quub  
- A general-purpose bridge  
- Substrate / Cosmos / SVM / a new VM  
- PII or full ISO documents on-chain  
- `quub_` RPC in year 1  
- Superchain registry on day one  
- Calling Chainlink from a precompile  
- Anvil as Mode A L1  
- Mode B in the same binary as Mode A  

---

## 11. How to say it

Quub sits beside Base and SWIFT. The dollar stays Circle’s dollar. Ops talk to `operator-api`. Year-1 the ledger **records** who paid whom and whether policy allowed it **before** CCTP moves USDC. Fees are a listed dummy on `--dev`, not a coin. Security is local Mode A now and Ethereum DA when a client asks for a public L1. There is no Quub coin to buy.
