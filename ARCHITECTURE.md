# Quub — Locked architecture
**Date:** 15 September 2026  
**Status:** binding for engineers, Cursor, and design partners  
**Network:** Quub  
**Company:** fazeZERO (legal entity only; customers and code hear Quub)

There is one product. Quub is a payments blockchain. Gateway, ISO, Evidence, and Policy Engine are Quub components, not a second brand. There is no native token.

Sites: `quub.network` = protocol. `quub.fi` = Gateway / operator UI.

---

## 0. What Quub is

A payment settlement layer that a licensed institution can contract.

It does **not** replace Ethereum, Solana, Base, Tempo, Arc, SWIFT, or a bank core.  
It does **not** issue the dollar.  
It does **not** require anyone to buy a chain coin.

It does:

- decide whether a payment may move (policy)
- commit ISO payment identity on the receipt (memo)
- take gas in a listed stable
- reserve a lane so junk cannot eat payroll
- optionally settle the payment on Quub Chain
- exit native USDC / other official assets onto the rail the counterparty already uses

---

## 1. Architecture decisions (locked)

| ID | Decision | Why | Rejected |
|---|---|---|---|
| **ADR-1** | One product: Quub | Two brands confused the team and the client | xZERO as a sister product |
| **ADR-2** | No native / gas token | Banks will not buy a thin float to move a dollar. Base and Tempo already proved payments can price blockspace in stables | QUUB / QUB / QUBE / QUBIC ticker, token-weighted validators |
| **ADR-3** | Rust node, EVM execution, Solidity money contracts | Circle, Fireblocks, Visa, bank counsel, Foundry, Alloy live here. Tempo/Arc/Base all chose this | Substrate/FRAME, Cosmos SDK, SVM, Move, custom VM, JAM, PolkaVM |
| **ADR-4** | Reth as a **library** (see ADR-016 for the live pin) | Do not maintain a Quub fork of Reth | Forking Reth, floating `main`, overriding `revm` |
| **ADR-5** | Default consensus = **Mode A** OP Stack rollup | Ethereum security + blob DA. Sequencer is a licensed operator | Public token-weighted L1, Tendermint, BABE/GRANDPA, Solana consensus |
| **ADR-6** | Optional consensus = **Mode B** Commonware Simplex | Only if named FIs fund 2f+1 validators. Same `quub-evm` crate | Shipping both modes in one binary |
| **ADR-7** | Year-1 local node = `quub-node --dev` + `EthereumNode` + custom executor | Shortest path to a block. `op-node` is Sprint 2 | Starting Simplex or `op-node` in Sprint 1 / 1.5 |
| **ADR-8** | Precompiles for policy / memo / paymaster | Protocol-cost gates. Cannot be skipped by calling the token a different way | Application-only policy |
| **ADR-9** | Frozen addresses F201–F203, F210–F213 | Stable for wallets, Foundry, and Gateway | Tempo-style prefixes (`0x20fc`…) |
| **ADR-10** | No PII on-chain | Supervisors read files; the chain stores hashes | Full `pacs.008` XML in calldata |
| **ADR-11** | JSON-RPC is `eth_*` | Fireblocks / MetaMask / Alloy / Foundry work unchanged | `quub_` namespace on by default; new EIP-2718 type in year 1 |
| **ADR-12** | Multi-chain by **adapters + official transports**, not a hub wrap | Circle owns USDC teleport. Chainlink owns general messaging | Quub-canonical wrapped dollar; “support all chains” as a lock-and-mint |
| **ADR-13** | USDC hops = **CCTP V2**. Other tokens / messages = **CCIP**. FX quotes = Data Streams | Do not rebuild Chainlink or Circle | CCIP-wrapping USDC when CCTP exists; calling oracles from F201/F203 |
| **ADR-14** | Quub is a settlement *option*, not the only venue | Counterparties already live on Base / Solana / Tempo | Forcing every creditor onto Quub |
| **ADR-15** | Spell Q-U-U-B. Never Qubic | Live L1 collision | Tickers QUB, QUBE, QUBC, QUBIC |
| **ADR-16** | Execution pin: `op-rs/reth` @ `aef8d3ef…` + optimism `op-reth/v2.4.4` + op-node **v1.19.7**; one Reth remote | Matches what op-reth/v2.4.4 ships; unlocks Mode A | `paradigmxyz/reth` v2.5.2 (no optimism crates); two Reth remotes; inventing portal / hand-rolled rollup.json |
| **ADR-17** | Payment lane: F210 + `transferWithMemo` only; `fill_lanes` 70/30 + spill; lane beats tip; same fill on `--dev` and `--engine` | Reserve payment blockspace without a new EIP-2718 type | Tip-only packing; PolicyAdmin 70/30; plain `transfer` as payment; Reth bump for custom envelopes |
| **ADR-19** | Paymaster live: memo-only fee via F213/`paymasterDebit`; freeze instant; unfreeze/params other-owner confirm; `--dev` OwnerB=anvil1 | Fee moves + maker-checker without timelock / 4337 | Instant unfreeze; EOA `takeFee`; fee on plain transfer; `tx.gasprice` quote |
| **ADR-20** | First public rail = CCTP V2 Sepolia→Base Sepolia; Quub 8091 records only; Circle burns/mints USDC; mock valid offline | Official USDC hop without Quub wrapping | Quub as CCTP domain; burn id = Quub tx; CCIP for USDC; Solana this sprint |

Full text: [`docs/adr/ADR-016-execution-pin.md`](docs/adr/ADR-016-execution-pin.md).  
Full text: [`docs/adr/ADR-017-payment-lane.md`](docs/adr/ADR-017-payment-lane.md).  
Full text: [`docs/adr/ADR-019-paymaster-dual-control.md`](docs/adr/ADR-019-paymaster-dual-control.md).  
Full text: [`docs/adr/ADR-020-cctp-base-rail.md`](docs/adr/ADR-020-cctp-base-rail.md).

---

## 2. System context

```
 Originating systems          Public rails              Bank rails
 ERP · PSP core · pain.001    Solana Base Tempo Arc     SWIFT · RTR · Aani
         │                    Ethereum Tron             Fedwire / Lynx
         │                         │                         │
         └─────────────┬───────────┴─────────────┬───────────┘
                       ▼                         ▼
                 Quub Gateway              Official transports
                 (the only API)            CCTP (USDC)
                       │                   CCIP (other + messages)
          ┌────────────┼────────────┐
          ▼            ▼            ▼
   Policy Engine    Quub ISO    Quub Evidence
          │            │            │
          └────────────┼────────────┘
                       ▼
                  Quub Node
                  F201 F202 F203
                  F210–F213 + lane
                       │
            Mode A: Ethereum blobs
            Mode B: named FI Simplex (later)
```

Actors: licensed PSP / EMI, bank treasury, stable issuer, corridor LP, supervisor.  
Quub does not custody client keys (Fireblocks / HSM). Quub does not run a SWIFT switch.

---

## 3. Containers

### On-chain (consensus-critical)

| Component | Address | Role |
|---|---|---|
| Quub Policy | `0x…F201` | allow / deny / freeze / threshold / pause. Fail closed. Caller must be F210 |
| Quub Memo | `0x…F202` | ISO identity hash. No PII, no timestamp |
| Quub Paymaster | `0x…F203` | quote + (later) takeFee in a listed stable |
| Quub Payment Token | `0x…F210` | the dollar / CAD / AED that moves. Not a gas coin |
| Quub Policy Admin | `0x…F211` | owners, freeze map, pause, threshold, fee token |
| Quub Evidence Anchor | `0x…F212` | `packHash` of the off-block file |
| Quub Fee Entry | `0x…F213` | Solidity wrapper over F203 |
| Quub Lane | mempool + payload | 70% of block gas reserved for real payments (Sprint 3) |

Chain ids (placeholders, confirm on chainid.network before freeze): mainnet `8090`, `--dev` / testnet `8091`.

### Node-adjacent (still Quub; cannot live in a public block)

| Component | Role |
|---|---|
| Quub Gateway | Client API. Route table. CCTP / CCIP / native RPC / Quub Node / ISO file out |
| Quub Policy Engine | Same `check()` as F201, for pre-check and public-rail hops |
| Quub ISO | `pacs.008` / `pain.001` ↔ F202 fields |
| Quub Evidence | Full ISO file + approvals + Travel Rule pack. Locator → F212 |

Repo (target names):

```
quub/
  crates/   quub-primitives quub-precompiles quub-evm quub-pool
            quub-payload quub-node quub-consensus-op quub-consensus-simplex quub-rpc
  services/ quub-policy quub-iso quub-evidence quub-gateway
  contracts/
  specs/
```

`services/xzero-*` still on disk are a **rename commit after Sprint 1.5**, not a second product.

---

## 4. Runtime paths

**A — public rail (Phase 0, most volume)**  
Client → Gateway → Policy Engine → ISO map → Evidence store → USDC on Base / Solana / Tempo via CCTP or native RPC → `pacs.008` out.  
Quub Chain is not in the path.

**B — settle on Quub (Phase 2, after 1.5 works)**  
Same intake. `transferWithMemo` on F210 → F201 `sload`s F211 → move → F202 memoHash → F212 if `packHash != 0` → F203 gas in listed stable.

**C — enter / leave Quub**  
CCTP for native USDC. CCIP for non-USDC and messages. Official issuer path for CAD/AED stables. No wrap as the unit of record.

**D — fiat last mile**  
Quub ISO emits `pacs.008` to SWIFT / RTR / Aani. The chain never speaks SWIFT.

---

## 5. Consensus

### Mode A — default

Optimistic rollup, OP Stack.

- Sequencer: licensed operator (fazeZERO or a design partner). Orders transactions. Not a public validator set.
- Agreement: `op-node` derives L2 from Ethereum. DA = Ethereum blobs.
- Execution: `quub-node` + `QuubEvmFactory` + F201–F203.
- Finality: sequencer soft confirmation, then Ethereum (+ fault-proof window for trust-minimized exit).
- Sprint 1 / 1.5: **no `op-node`**. `--dev` uses Reth `EthereumNode` so engineers can mine a block. Mode A types swap is **Sprint 2**.

### Mode B — optional

Commonware Simplex BFT. Named FI set, 2f+1, ~0.5s finality. Same `quub-evm`. Feature flag `mode-b`. Never enable `mode-a` and `mode-b` in one binary. Compiling stub until a consortium funds validators.

### Not used

PoW, Ethereum L1 as *our* validator set, Tendermint, Substrate consensus, Solana Tower, token-weighted anything.

---

## 6. Execution and precompiles

- Pin Reth per **ADR-016** (`op-rs/reth` @ `aef8d3ef92117f91455e16969f0adf5bf7c6e9e1`). Alloy on Reth crates follows that pin (~1.6 / alloy-evm ~0.37). Workspace Alloy 0.8 stays on services until a dedicated bump. Convert addresses only in `quub-evm` wrap.
- Inject F201–F203 when `spec >= PRAGUE` (not `==`).
- F201 and F202: `DynPrecompile::new_stateful` (F201 reads F211 storage; F202 hash includes `tx.origin`, which is not in calldata). F203 `quote` may stay cacheable until `takeFee` writes state.
- `quub-precompiles` has **no** `reth-*` dependency. `quub-evm` is the only crate that `sload`s.
- F201 production path: `internals_mut().sload(F211, …)` → `quub_policy::check`. Empty F211 → deny. Caller must be F210.
- Storage slots from `forge inspect`, not from reading Solidity by eye. Expected PolicyAdmin packing: slot 0 `ownerA`; slot 1 `ownerB` + `paused` at byte 20; slot 2 `threshold`; slot 3 `feeToken`; slot 4 `frozen` base.

### F202 hash (locked)

```
memoHash = keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, tx.origin))
```

No timestamp. No PII. Same formula in `quub-iso`, F202 wrap (`tx_origin()`, not `caller`), PaymentToken, Foundry shim.  
`usd_pacs008_stable_hash` uses a **fixed** origin `0x…00AA`. A mined `--dev` payment uses the anvil EOA `0xf39F…` — different origin, different hash. Both are correct.

---

## 7. Interop and Chainlink

| Job | Owner | Quub |
|---|---|---|
| Native USDC hop | Circle CCTP V2 | Gateway calls it |
| Other tokens + messages | Chainlink CCIP | Gateway calls it |
| Bank ISO → many chains | SWIFT + CRE | Quub is a destination + policy box |
| FX / NAV | Chainlink Data Streams | Gateway reads. Never inside F201/F203 |
| Who may pay, identity, evidence, lane, dedicated settlement | Quub | The product |

Year-1 adapters: Base, Solana, Ethereum, Tempo, CCTP, CCIP, Quub Node, ISO file out.  
Add a chain only when a licensed client has volume **and** an official asset path exists. Otherwise Gateway refuses.

---

## 8. Deployment

| Topology | What runs | Production money |
|---|---|---|
| **A — Gateway only** | Gateway + Policy + ISO + Evidence on `quub.fi`, public RPCs | Base / Solana / Tempo |
| **B — Mode A** | + `quub-node` sequencer + `op-node` + Ethereum blobs | Selected corridors on Quub; CCTP exit |
| **C — Mode B** | Same Gateway and contracts; Simplex + named FIs | Only if those FIs fund it |

`--dev` (8091) is engineers only. Anvil keys are public. OwnerA = anvil0, OwnerB = anvil1 on `--dev` (ADR-019). Mode A L2 alloc may still share owners — Foundry owns the checker proof there (`STATUS.md`). Production 24h timelock is later.

Never in the `quub-node` image: sequencer keys, vendor secrets, full ISO XML.

---

## 9. Build sequence

| Sprint | Outcome | Status |
|---|---|---|
| **0** | Policy / ISO / Evidence libs + Foundry facades | Green (13 Rust + 6 Foundry) |
| **1** | `quub-node --dev`, F201–F203 registered, classifier | Green |
| **1.5** | Genesis F210–F213, F201 reads F211, one mined `transferWithMemo` | **Next** |
| **2** | Mode A: `op-node` + Engine API, one deposit, one L2 block | After 1.5 |
| **3** | Payment-lane 70/30 fill | After 2 |
| **4** | Mode B stub compiles against the same evm crate | After 2 |
| **5** | Gateway against `quub-node` and public Base | Product path |

Kill criteria (unchanged): no 3 licensed clients in 12 months → no public mainnet. Token launch becomes the plan → abort. Strategy is out-TPS Solana → abort.

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
- Competing with CRE on SWIFT orchestration for 600 banks

---

## 11. How to say it

Quub sits beside the rails money already uses. Gateway is the control point. The ledger is an optional settlement zone when the client needs policy, ISO identity, and a payment lane in the same transaction. Fees are in the stable being moved. Security is Ethereum (Mode A) or a named FI set (Mode B). There is no Quub coin to buy.
