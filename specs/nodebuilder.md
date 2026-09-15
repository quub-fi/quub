# Quub NodeBuilder Sketch
**Network name:** quub (working)  
**Company / orchestration:** fazeZERO / xZERO — do not rename  
**Status:** implementation pack, not a whitepaper  
**Date:** 15 September 2026  
**Pin window:** Reth 2.5.x / op-reth 2.4.x / Commonware consensus current  
**Rule:** one execution crate. Consensus is a feature flag. Do not fork Reth.

Quub is the ledger. xZERO stays the off-chain orchestration, ISO mapper, and evidence plane. Architecture is unchanged from the prior sketch; only names move.

### Naming rules
| Surface | Value |
|---|---|
| Prose | Quub |
| Code, crates, binary | `quub`, `quub-node` |
| RPC | `eth_*` unchanged. Optional `quub_` namespace, off by default |
| Chainspec | `quub`, `quub-devnet`, `quub-sepolia` (Mode A), `quub-local` |
| Chain id (placeholder) | mainnet `8090`, testnet `8091` — confirm unused on chainid.network before freeze |
| Client ident | `quub/v0` |
| Native token | none. Never ticker QUB, QUBE, QUBC, or QUBIC |
| Payment token standard | internal label QUUB-20 if needed; public name is “payment token” |

### Collisions — working name is fine, public letterhead is not
1. **Qubic / $QUBIC** (qubic.org) — live L1. Spoken near-twin. Search for “quub blockchain” will hit them. Always spell Q-U-U-B. Never say “the Qubic network.”
2. **quub.fi** — exact string, DeFi risk-monitoring site. Do not use `.fi`.
3. **Quub Inc. / quub.space** — Lancaster PA satellite company, already pronounced “cube.” US software/data trademark search required before any public brand spend.
4. crates.io: no `quub` crate today. Reserve `quub`, `quub-node`, `quub-evm` now.

Pronunciation in the vendor pack: confirm with Nasser. The satellite company already claims “cube.” Public copy should say “Quub payments fabric” until counsel clears the exact mark.

This is the artifact that follows the stack decision: Rust node, EVM state, Mode A (OP Stack rollup) default, Mode B (Simplex L1) only if a named FI consortium funds validators.

---

## 0. What you build this week vs. what you do not

Build:

- Workspace + `quub-evm` crate that injects three precompiles into a Reth `EvmConfig`
- Payment-lane transaction pool (classifier only; no exotic ordering)
- Binary that launches as either `op-reth`-style Engine API follower **or** Simplex-driven sequencer
- Foundry suite that talks to the three precompiles as if they were contracts

Do not build this week:

- A token
- A bridge
- A custom VM
- FRAME / Substrate
- JAM
- A fee AMM (wrap an existing paymaster; copy Tempo later if needed)
- Stylus
- zk proving (leave `quub-prover` as an empty crate)

---

## 1. Workspace

```
quub-node/
  Cargo.toml
  rust-toolchain.toml          # pin the Reth 2.5 nightly/stable that op-reth 2.4.x uses
  crates/
    quub-primitives/          # chain id, precompile addresses, TIP-style types
    quub-evm/                 # EvmConfig + PrecompilesMap  ← SHARED
    quub-pool/                # payment-lane TxPool
    quub-payload/             # payload builder (payment lane fill policy)
    quub-node/                # NodeTypes + NodeBuilder wiring
    quub-consensus-op/        # Mode A: Engine API / op-node adapter    [feature = "mode-a"]
    quub-consensus-simplex/   # Mode B: Commonware Simplex automaton    [feature = "mode-b"]
    quub-precompiles/         # policy, iso_memo, paymaster
    quub-bin/                 # quub-node binary
  contracts/                   # Foundry: token, policy facade, tests
  prover/                      # empty stub, SP1 guest later
```

`quub-evm`, `quub-pool`, `quub-precompiles` compile under **both** features.  
Only `quub-consensus-*` and the binary entrypoint change.

Default features: `mode-a`. Never enable both in one binary.

```toml
# crates/quub-node/Cargo.toml
[features]
default = ["mode-a"]
mode-a = ["dep:quub-consensus-op"]
mode-b = ["dep:quub-consensus-simplex"]
```

---

## 2. Crates to pin

Pin by **git rev**, not crates.io, for anything in the Reth / OP / Commonware family. Tempo and Base both pin Reth commits. Follow that.

| Crate | Role | Pin source (Sept 2026) |
|---|---|---|
| `reth-node-builder` 2.5.x | `NodeBuilder`, `ComponentsBuilder`, `AddOns`, ExEx | `paradigmxyz/reth` tag aligned with Reth 2.5.2 |
| `reth-node-api` | `NodeTypes`, engine types | same rev |
| `reth-evm` | `EvmConfig`, `PrecompilesMap`, `DynPrecompile` | same rev |
| `reth-revm` / `revm` | execution | whatever that Reth rev carries (do not override) |
| `alloy` / `alloy-primitives` / `alloy-sol-types` | addresses, ABI, RPC | Reth workspace alloy version |
| `reth-transaction-pool` | custom pool | same rev |
| `reth-payload-builder` | custom payload | same rev |
| `reth-optimism-node` | Mode A node types, deposit tx, OP engine | `ethereum-optimism/optimism` `op-reth/v2.4.x` |
| `reth-optimism-evm` | OP `EvmConfig` to **extend**, not replace | same |
| `reth-optimism-txpool` | starting point for payment lane | same |
| `reth-optimism-payload-builder` | starting point for payload | same |
| `op-alloy-*` | OP genesis / deposit types | whatever op-reth 2.4.x uses |
| `commonware-consensus` | Mode B Simplex | `commonwarexyz/monorepo` current simplex |
| `commonware-cryptography` | BLS / ed25519 validator keys | same |
| `commonware-p2p` / `commonware-runtime` | Mode B networking | same |
| `eyre`, `tokio`, `tracing`, `clap` | binary | match Reth |

Do not take a newer `revm` than the Reth rev. That is how custom EVMs break.

Reth examples to copy, not rewrite:

- `examples/custom-evm` — precompile injection
- `examples/custom-node-components` — pool + payload swap
- `examples/custom-payload-builder`
- Base pattern: extend `reth-optimism-node` via `NodeBuilder`, do not fork Reth

---

## 3. Addresses (genesis constants)

Fixed. Same on every network. Document in `quub-primitives`.

Do **not** use Tempo prefixes (`0x20fc`, `0xfeec`, `0x403c`, `0xdec0`, `0x20c0`). Those are guarded in their specs. Use a distinct `0xF2` namespace so explorers and counsel never think this is a Tempo fork.

```
# precompiles (hot path, consensus-critical)
QUUB_POLICY          0x000000000000000000000000000000000000F201
QUUB_ISO_MEMO        0x000000000000000000000000000000000000F202
QUUB_PAYMASTER       0x000000000000000000000000000000000000F203

# reserved, do not implement week 1
FEE_MANAGER          0x000000000000000000000000000000000000F204
LANE_METER           0x000000000000000000000000000000000000F205
```

System contracts (Solidity, upgradeable behind a 24h timelock + dual-control, **not** precompiles):

```
PAYMENT_TOKEN        0x000000000000000000000000000000000000F210
POLICY_ADMIN         0x000000000000000000000000000000000000F211
EVIDENCE_ANCHOR      0x000000000000000000000000000000000000F212
PAYMASTER_ENTRY      0x000000000000000000000000000000000000F213
```

Year-1 rule: precompiles evaluate; Solidity holds mutable policy / fee-token lists. Upgrading a denylist is a contract tx, not a hardfork. Promoting `check` storage into the precompile is a later hardfork if gas demands it.

---

## 4. NodeBuilder (shared)

```rust
// crates/quub-node/src/lib.rs
use reth_node_builder::{NodeBuilder, NodeConfig, components::ComponentsBuilder};
use quub_evm::QuubEvmConfig;
use quub_pool::QuubPoolBuilder;
use quub_payload::QuubPayloadBuilder;

#[cfg(feature = "mode-a")]
use quub_consensus_op::QuubOpNode;

#[cfg(feature = "mode-b")]
use quub_consensus_simplex::QuubSimplexNode;

pub fn launch(config: NodeConfig) -> eyre::Result<()> {
    let evm = QuubEvmConfig::mainnet(); // injects the three precompiles

    let components = ComponentsBuilder::default()
        .node_types::<QuubNode>()
        .pool(QuubPoolBuilder::default())
        .payload(QuubPayloadBuilder::default())
        .executor(evm);

    #[cfg(feature = "mode-a")]
    {
        // Engine API on authrpc. op-node (Go) is the consensus process.
        NodeBuilder::new(config)
            .with_types::<QuubOpNode>()
            .with_components(components)
            .launch_with_engine()?;
    }

    #[cfg(feature = "mode-b")]
    {
        // Single binary. Reth runs. Simplex runs on a second tokio runtime
        // and drives payload build / commit through the Automaton trait.
        NodeBuilder::new(config)
            .with_types::<QuubSimplexNode>()
            .with_components(components)
            .on_component_started(|ctx| {
                quub_consensus_simplex::spawn(ctx)?;
                Ok(())
            })
            .launch()?;
    }

    Ok(())
}
```

`QuubEvmConfig` is the only place precompiles are registered. Mode A and Mode B both call it. If a precompile behaves differently across modes, you have already forked the product.

Mode A runtime processes: `op-node` + `quub-node` (op-reth-shaped).  
Mode B runtime process: `quub-node` only (Tempo-shaped: Reth thread + Commonware thread).

---

## 5. EvmConfig: inject three precompiles

```rust
// crates/quub-evm/src/lib.rs
use reth_evm::precompiles::{DynPrecompile, PrecompilesMap};
use quub_precompiles::{iso_memo, paymaster, policy};

impl QuubEvmConfig {
    pub fn precompiles(&self) -> PrecompilesMap {
        let mut map = PrecompilesMap::from_osaka(); // whatever hardfork you declare
        map.apply(|set| {
            set.move_precompile(/* do not collide with 0x01–0x11 builtins */);
        });
        map.extend([
            (policy::ADDRESS,     DynPrecompile::new(policy::run)),
            (iso_memo::ADDRESS,   DynPrecompile::new(iso_memo::run)),
            (paymaster::ADDRESS,  DynPrecompile::new(paymaster::run)),
        ]);
        map
    }
}
```

Copy `examples/custom-evm` for the `ConfigureEvm` impl. Start from `reth-optimism-evm` if Mode A is on, then **add** the three addresses. Do not drop OP deposit / L1-fee precompiles.

---

## 6. Precompile 1 — Policy (`0x…F201`)

**Job:** answer “may this transfer happen?” before the token commits.  
**Does not:** own the policy set. `POLICY_ADMIN` (`0x…F211`) is source of truth. Year-1 the precompile **staticcalls** F211, then returns a reason code. Promote to native precompile storage only if gas or atomicity demands it.

```solidity
// called only by PAYMENT_TOKEN._beforeTransfer (caller allowlist = F210)
function check(
    address token,
    address from,
    address to,
    uint256 amount,
    bytes32 trHash          // Travel Rule commitment from Notabene / equivalent
) external view returns (bool allowed, uint16 reason);
```

Reason codes (frozen, never reuse):

| code | meaning |
|---:|---|
| 0 | allow |
| 1 | frozen_from |
| 2 | frozen_to |
| 3 | not_allowlisted |
| 4 | amount_over_limit |
| 5 | missing_travel_rule |
| 6 | dual_control_required |
| 7 | paused |
| 8 | malformed |

**Gas:** 3_000 + 2_000 if `trHash != 0`.  
**Failure:** malformed input → `allowed = false`, reason 8. The token emits `PolicyRejected` and reverts. Fail closed. Unknown selector → revert empty.  
**Do not** put OFAC lists in the node binary. Dual-control on F211 (2-of-3 HSM, 24h timelock) updates freezes and thresholds.  
**Caller lock:** only `F210` may call `check` in year 1. Users talk to the token, not to the precompile.

---

## 7. Precompile 2 — ISO 20022 memo (`0x…F202`)

**Job:** validate the *identity set* of a payment and return a commitment. Full `pacs.008` / `pain.001` XML stays in the xZERO evidence plane. Nothing that looks like a name, IBAN, or address goes on-chain.

```solidity
function validateAndCommit(
    bytes32 endToEndId,   // keccak256(UTF-8 EndToEndId), preimage ≤ 35 chars off-chain
    bytes16 uetr,         // UUIDv4
    bytes32 instrId,      // keccak256(UTF-8 InstrId)
    bytes3  ccy,          // ISO 4217, e.g. "USD" / "CAD" / "AED"
    uint8   msgType       // 0=pacs.008 1=pain.001 2=pacs.009 3=camt.054
) external view returns (bytes32 memoHash);
```

Rules:

- `endToEndId != 0`
- `msgType ∈ {0,2}` ⇒ `uetr != 0` (interbank)
- `ccy ∈ {USD, CAD, AED, SAR}` year-1; list lives on `POLICY_ADMIN`, not hardcoded in Rust after genesis
- `memoHash = keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, tx.origin))`  
  Do **not** put `block.timestamp` in the hash. Treasurers need a stable id.

**Gas:** 2_500.  
**Failure:** rule miss → revert. No best-effort parse.  
**Storage:** precompile stores nothing. `PAYMENT_TOKEN` emits `MemoAnchored(bytes32 memoHash, uint8 msgType)` and `EVIDENCE_ANCHOR` stores the off-chain pack hash.  
**Canonicalization doc** (`contracts/iso-memo.md`) is part of consensus. Two PSPs hashing the same payment differently means the precompile failed.

---

## 8. Precompile 3 — Paymaster hook (`0x…F203`)

**Job:** quote and collect gas in a stablecoin so there is no volatile gas token.  
**Does not:** be an ERC-4337 bundler, a Fee AMM, or a new EIP-2718 tx type. Those are Phase 2 (Tempo `0x76` / TIP-20 fee path). Year-1 envelope stays EIP-1559 + a Solidity paymaster at `F213` that calls this hook.

```solidity
function quote(address feeToken, uint256 gasLimit, uint256 gasPrice) external view returns (uint256 tokenAmount);
function takeFee(address payer, address feeToken, uint256 tokenAmount) external;
```

Rules:

- `takeFee` reverts unless caller is the protocol host or `PAYMASTER_ENTRY` (`F213`). Users never call it.
- `feeToken` must be on the allowlist in `POLICY_ADMIN`. Genesis: one USDC address per network.
- Rate is a 1e6-scale posted price in extra-data / a system tx. **Do not call Chainlink inside the precompile.**
- If `payer` cannot pay, the pool must drop the tx *before inclusion*. Include-and-revert griefs the payment lane.
- Quote is deterministic given `(feeToken, gasLimit, gasPrice, posted rate)`. Predictability beats optimality. No AMM week 1.

**Gas of the hook:** 8_000 including the system debit.  
**Receipt:** surface `feeToken` + `feePayer` + `tokenCharged` so recon files match Tempo-style receipts later.

---

## 9. Payment lane (pool + payload)

Do not invent a new tx type week 1. Classify existing txs. Prefix-only classifiers are how junk calldata steals reserved gas (Tempo hit this). Match selector **and** expected ABI length.

```rust
fn is_payment(tx: &TxEnvelope) -> bool {
    let to = match tx.to() { Some(t) => t, None => return false };
    if !is_registered_payment_token(to) { return false; }
    matches_selector_and_len(tx.input(), &[
        transfer,              // 68 bytes
        transferFrom,          // 100 bytes
        transferWithMemo,      // 68 + memo head
        transferFromWithMemo,
    ])
}
```

Block gas split (genesis, governance later):

```
PAYMENT_LANE_GAS    70% of block gas limit
GENERAL_LANE_GAS    30% of block gas limit
```

Hook points in Reth (copy, do not rewrite the pool):

1. `PoolBuilder::build_pool` — wrap the validator with `PaymentLaneValidator`; tag the tx.
2. Custom `TransactionOrdering` — payment lane is FIFO among fee-prepaid txs; general lane is coinbase-tip.
3. `PayloadServiceBuilder` — two-pass fill: payment quota first, then general. General cannot consume reserved gas.

No priority-fee auction inside the payment lane. First-seen, valid, fee-prepaid. Custom `lane` field on a new envelope is Phase 2.

---

## 10. Mode A vs Mode B — what actually differs

| Surface | Mode A (default) | Mode B |
|---|---|---|
| Execution crate | `quub-evm` | same |
| Precompiles | same addresses, same code | same |
| Pool / payload | same | same |
| JSON-RPC | standard `eth_*` | same |
| Consensus process | `op-node` (Go) via Engine API / JWT | Commonware Simplex in-process |
| DA | Ethereum blobs | chain itself |
| Finality | L2 soft instant; L1 economic after proposal | deterministic ~0.5s |
| Genesis extra | OP deposit / L1 attributes | Simplex validator set + epoch |
| Binary feature | `--features mode-a` | `--features mode-b` |

If you need a field on `NodeTypes` that is mode-specific (engine payload attributes), put it in the consensus crate, not in `quub-evm`.

---

## 11. Bank / vendor pack (what sits next to the crates)

List this in the same repo under `/vendor-pack` so counsel and a Fireblocks engineer can read it without opening Rust.

- Chain id, genesis hash, hardfork calendar (declare Osaka-class and freeze it)
- Precompile address table + ABI JSON
- Policy codes table
- ISO schema ids and canonicalization doc
- Fee token address + quote formula
- Sequencer / validator key ceremony: secp256k1 for Engine API JWT (Mode A), Commonware keys in HSM (Mode B)
- Dual-control on `POLICY_REGISTRY` admin: 2-of-3, 24h timelock, HSM
- What Chainalysis / Notabene / Fireblocks see: standard ERC-20 Transfer logs plus `MemoAnchored(bytes32,uint16)` and `PolicyChecked(uint16)` events from the facades
- Evidence plane: off-chain pack hash written to `EVIDENCE_ANCHOR`
- Explicit non-goals: no native speculative token, no opaque upgrade proxy on precompiles (precompiles change only via node release + hardfork)

---

## 12. Test bar before any public testnet

Foundry (`contracts/`):

1. USDC-like token `transferWithMemo` → ISO precompile succeeds, receipt has memo hash  
2. Frozen `from` → policy code 2, token balance unchanged  
3. Amount above threshold, empty attestation → code 3  
4. Paymaster: account with zero fee-token balance never lands in a block  
5. Payment-lane fill: 100 payment txs + 100 junk txs, block contains payments up to 70% gas  

Rust (`quub-precompiles`):

6. Canonical memo hash is stable across two independent implementations (orchestrator + precompile)  
7. Unknown selector reverts  
8. Precompile addresses do not collide with `0x01–0x11`

Mode flag:

9. `cargo test -p quub-evm --features mode-a` and `--features mode-b` produce the same precompile bytecode / gas schedule

---

## 13. Hard rules

1. Do not fork Reth. Depend and extend.
2. Do not let Mode B grow a second EVM config.
3. Do not put sanctions lists or ISO full documents in the node.
4. Do not ship a token from this repo.
5. Do not enable `mode-a` and `mode-b` in one binary.
6. Precompile behaviour is consensus-critical. Treat a gas-schedule change as a hardfork.
7. If Circle / Fireblocks cannot point at a normal `eth_getLogs` Transfer, you have failed the architecture.

When this pack is implemented, the next artifact is the Foundry facade + `transferWithMemo` reference token, not a consensus rewrite.
