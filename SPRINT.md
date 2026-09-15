# Quub — Sprint 1 implementation direction
**Status of Sprint 0:** accepted. 13 Rust + 6 Foundry tests is the right bar. Do not reopen those crates except to share types.  
**This document is what Cursor executes next.** Put it in the repo as `SPRINT.md` and paste the first message at the bottom.

---

## What “full implementation” means, sequenced

| Sprint | Outcome | Consensus | Stop when |
|---|---|---|---|
| **0** | xZERO libs + Foundry facades | none | `cargo test --workspace` + `forge test` |
| **1** | `quub-node --dev` with F201–F203 live | none (Reth `--dev` / EthereumNode) | one `transferWithMemo` mined on the local node |
| **1.5** | Genesis alloc of F210–F213 + Foundry script against that node | none | `forge script` on `http://127.0.0.1:8545` green |
| **2** | Mode A: OP types + Engine API + `op-node` local | OP Stack | deposit tx + L2 block |
| **3** | Payment-lane pool + payload fill 70/30 | still Mode A | junk calldata cannot steal payment gas |
| **4** | Mode B stub compiles, not shipped | Simplex behind flag | `cargo test -p quub-evm --features mode-b` same precompile set |
| **5** | Operator API + xZERO against `quub-node` and public Base | — | one licensed-client happy path |

Sprint 1 does **not** include: `op-node`, Commonware, a bridge, a token, proxies, timelocks, `apps/operator-api`, production chain ids.

Reason: Reth’s own `examples/custom-evm` launches an `EthereumNode` with a swapped `ExecutorBuilder`. That is the shortest path to a block. Mode A (OP) is a types swap in Sprint 2, not a rewrite.

---

## Locked additions for Sprint 1 (do not debate)

1. Pin **`reth-ethereum`** from `paradigmxyz/reth` **2.5.x** (docs current: 2.5.2) via git tag, features `["node", "evm", "cli"]`. Do not add `reth` meta + ten crates by hand if `reth-ethereum` already re-exports them.
2. Copy the pattern in `reth/examples/custom-evm`: `EvmFactory` + `ExecutorBuilder` + `EthereumNode::components().executor(...)`.
3. Precompile **bodies** live in `quub-precompiles` and **call** `xzero-policy` / `xzero-iso`. Do not duplicate logic. Solidity `PolicyAdmin` remains source of truth for freezes; F201 year-1 may evaluate the same rules in-process (see §3).
4. Feature flags stay as declared. Sprint 1 binary is `mode-a` default. `quub-consensus-op` can stay a compiling stub (`pub fn placeholder() {}`).
5. Chain id in `--dev`: `8091` (testnet placeholder). HTTP RPC `127.0.0.1:8545`.
6. No new tx type. No `quub_` RPC namespace.

---

## 1. Crate work, in this order

Do not start `quub-node` until `quub-precompiles` and `quub-evm` unit-test.

### 1.1 `crates/quub-precompiles` (replace DEFERRED.md)

```
crates/quub-precompiles/
  Cargo.toml
  src/lib.rs
  src/policy.rs
  src/iso_memo.rs
  src/paymaster.rs
  src/abi.rs
```

`Cargo.toml` depends on: `quub-primitives`, `xzero-policy`, `xzero-iso`, `alloy-primitives`, `alloy-sol-types`.  
**No `reth-*` dependency.** That is what keeps Mode A and Mode B sharing this crate.

Each module exports:

```rust
pub const ADDRESS: Address = quub_primitives::QUUB_POLICY; // or ISO / PAYMASTER

pub fn run(input: &[u8], gas: u64, caller: Address) -> Result<(Bytes, u64), PrecompileError>;
```

Dispatch on the 4-byte selector from `abi.rs` (generate once with `sol!` in `abi.rs`).  
`policy::run` decodes `check(...)` and calls `xzero_policy::check`.  
`iso_memo::run` decodes `validateAndCommit(...)` and calls `xzero_iso::validate_and_commit`.  
`paymaster::run` implements `quote` as a pure function of `(token, gasLimit, gasPrice, posted_rate)` with the genesis USDC allowlist from `quub-primitives`. `takeFee` in Sprint 1 returns `Err` unless `caller` is the test system address; stateful debit waits until F213 is deployed on the node (Sprint 1.5).

Tests (add to the 13, do not break them):

- `policy_unknown_selector_reverts`
- `policy_reason_codes_match_xzero`
- `iso_hash_matches_xzero_iso`
- `paymaster_unlisted_token_errors`
- `addresses_are_f201_f202_f203`

### 1.2 `crates/quub-evm` (replace DEFERRED.md)

This is the only crate that talks to Reth EVM types.

```
crates/quub-evm/
  Cargo.toml          # reth-ethereum (evm), alloy-evm, quub-precompiles
  src/lib.rs
  src/factory.rs      # QuubEvmFactory
  src/precompiles.rs  # PrecompilesMap with F201–F203 inserted
```

Follow `reth/examples/custom-evm`:

```rust
impl EvmFactory for QuubEvmFactory {
    type Precompiles = PrecompilesMap;
    // create_evm: start from Prague/Osaka EthPrecompiles, then
    // map.apply_precompile(&QUUB_POLICY, |_| Some(DynPrecompile::new(policy::run)));
    // same for F202, F203
}
```

`QuubExecutorBuilder` implements `ExecutorBuilder` and returns  
`EthEvmConfig::new_with_evm_factory(ctx.chain_spec(), QuubEvmFactory)`.

Unit test: construct the map, assert `map.contains(F201)` / `F202` / `F203` and that `0x01` (ecrecover) is still present. Do not collide with `0x01–0x11`.

### 1.3 `crates/quub-pool` — classifier only

No Reth pool swap yet. Pure function + tests.

```rust
pub fn is_payment(to: Address, input: &[u8], registered: &HashSet<Address>) -> bool;
```

Selectors: `transfer`, `transferFrom`, `transferWithMemo`, `transferFromWithMemo`.  
Require exact ABI length. A `transfer` selector plus trailing junk is **not** a payment.

Tests: `transfer_exact_len_is_payment`, `transfer_prefix_plus_junk_is_not`, `unregistered_token_is_not`.

`quub-payload` stays DEFERRED until Sprint 3. Leave the file.

### 1.4 `crates/quub-node` — the binary

```
crates/quub-node/
  Cargo.toml          # bin = quub-node, features mode-a (default)
  src/main.rs
  src/cli.rs          # clap: --dev --http --http.port --chain-id
  src/launch.rs
```

`launch.rs` is the custom-evm example with names changed:

```rust
NodeBuilder::new(node_config)
    .with_types::<EthereumNode>()
    .with_components(EthereumNode::components().executor(QuubExecutorBuilder::default()))
    .with_add_ons(EthereumAddOns::default())
    .launch()
```

`--dev` uses Reth’s dev mode (instant blocks, funded dev key). Chain spec: Prague/Osaka activated, chain id `8091`.

`quub-consensus-op` / `quub-consensus-simplex` / `quub-rpc`: keep as compiling stubs. Do not implement Engine API or Simplex this sprint.

### 1.5 Workspace `Cargo.toml`

```toml
[workspace.dependencies]
reth-ethereum = { git = "https://github.com/paradigmxyz/reth", tag = "v2.5.2", default-features = false }

# pin alloy to the version that workspace of that tag uses
# if the tag's Cargo.toml has [workspace.dependencies] alloy = "x", copy that number
```

If `v2.5.2` does not fetch, use the latest `v2.5.*` tag that exists on the day you pin. Write the tag in `README.md`. Never float `main`.

---

## 2. How F201 talks to policy in Sprint 1

Two layers already exist. Do not merge them.

| Layer | Where | State |
|---|---|---|
| Solidity `PolicyAdmin` F211 | Foundry, deployed in 1.5 | freezes, thresholds, fee tokens |
| Rust `xzero-policy` | service crate | same rules, in-memory / test fixture |
| Precompile F201 | `quub-precompiles` | **calls `xzero-policy` in Sprint 1** |

Sprint 1 acceptance does **not** require F201 to staticcall F211 inside REVM. That is the year-1 production shape and it needs the contract deployed at a known address in the same state DB. Sequence:

- Sprint 1: F201 is a rust-native check (xzero-policy). Foundry tests still hit Solidity on anvil/local forge.
- Sprint 1.5: deploy F210–F213 onto `quub-node --dev`. Add one integration test: `PaymentToken.transferWithMemo` → Solidity `_beforeTransfer` can call F201 *or* F211. Pick **F211 for the token hook** (already works) and keep F201 callable for later protocol-native tokens.
- Later: token hook moves to F201 when you want protocol-cost checks.

Do not spend Sprint 1 on stateful precompiles + Solidity staticcall. That is how this week slips.

---

## 3. Sprint 1 acceptance tests (add these names)

Rust:

```
quub_precompiles::policy_reason_codes_match_xzero
quub_precompiles::iso_hash_matches_xzero_iso
quub_precompiles::paymaster_unlisted_token_errors
quub_evm::map_contains_f201_f202_f203_and_ecrecover
quub_pool::transfer_exact_len_is_payment
quub_pool::transfer_prefix_plus_junk_is_not
```

Manual / script (must be in `scripts/devnet.sh`):

```
# terminal 1
cargo run -p quub-node -- --dev --http --http.port 8545

# terminal 2
cast chain-id --rpc-url http://127.0.0.1:8545          # expect 8091
cast call 0x000000000000000000000000000000000000F202 \
  "validateAndCommit(bytes32,bytes16,bytes32,bytes3,uint8)(bytes32)" \
  0x1111... 0x2222... 0x3333... 0x555344 0 \
  --rpc-url http://127.0.0.1:8545
# must return a 32-byte hash, not revert-empty
```

If `cast call` to F202 reverts with empty data, the precompile is not registered. That is a Sprint 1 fail even if unit tests pass.

---

## 4. Sprint 1.5 (same week if 1 is green, else next)

1. Genesis or a Foundry script deploys `PolicyAdmin`, `PaymentToken`, `EvidenceAnchor`, `PaymasterEntry` to a **dev key**. Addresses do not have to be F210–F213 on `--dev` yet (CREATE will not land on those). Document actual addresses in `specs/devnet-addresses.md`.
2. `forge script` against `http://127.0.0.1:8545`.
3. One `transferWithMemo` mined; receipt contains `MemoAnchored`.
4. Frozen sender still reverts.

Only after that do you add a custom genesis alloc that **forces** F210–F213. That is a chainspec change and belongs in Sprint 2 with Mode A.

---

## 5. Sprint 2–5 map (direction only; do not start)

**Sprint 2 — Mode A types.** Swap `EthereumNode` for OP node types from `reth-optimism-node` (same Reth rev / `op-reth` 2.4.x). Keep `QuubEvmFactory`. Launch `op-node` in `--dev` against local L1 later; first milestone is the binary still serving `eth_*` with F201–F203 present under OP deposit handling. Do not drop L1 attributes / deposit tx support.

**Sprint 3 — payment lane.** Replace the default pool builder with `QuubPoolBuilder` wrapping the validator. Two-pass payload fill 70/30. Test: 100 payment + 100 junk, block gas of junk ≤ 30%.

**Sprint 4 — Mode B compile.** `quub-consensus-simplex` implements a thin Automaton that commits payload hashes. No public testnet. Gate: same precompile bytecode as Mode A.

**Sprint 5 — product.** `apps/operator-api` talks to xZERO orchestrator, which talks to `quub-node` **and** public Base. This is the first thing a licensed client sees. The chain is still an implementation detail.

---

## 6. Still forbidden

- Forking Reth
- Substrate / FRAME / Cosmos / SVM
- Native token, ticker, points
- New EIP-2718 type
- Fee AMM, Stylus, SP1
- Publishing `8090` as production
- Importing `github.com/quub-fi/quub-layer2`
- Putting keys, JWT, Fireblocks, Notabene in the repo
- Enabling `mode-a` and `mode-b` in one binary
- Changing F201–F203

---

## 7. Definition of done — Sprint 1

- [ ] `DEFERRED.md` removed from `quub-precompiles`, `quub-evm`, `quub-pool`, `quub-node`
- [ ] `cargo test --workspace` still includes the original 13 and the new tests in §3
- [ ] `forge test` still 6 green (do not break Foundry)
- [ ] `cargo run -p quub-node -- --dev --http` serves `eth_chainId = 8091`
- [ ] `cast call` to F202 returns a hash
- [ ] README documents the Reth git tag actually used
- [ ] No `op-node`, no Commonware in the running binary
- [ ] Agent printed the tree and the new test names, then stopped

---

## 8. First message to Cursor (paste this)

```
Sprint 0 is green. Do not touch passing xzero-* or Foundry tests except to depend on them.

Read AGENTS.md, STRATEGY.md, specs/nodebuilder.md, and SPRINT.md.

Execute Sprint 1 only:
1. Pin reth-ethereum to paradigmxyz/reth v2.5.x (write the exact tag in README).
2. Implement quub-precompiles by calling xzero-policy and xzero-iso. No reth dep in that crate.
3. Implement quub-evm QuubEvmFactory + QuubExecutorBuilder from reth/examples/custom-evm. Register F201 F202 F203. Keep 0x01–0x11.
4. Implement quub-pool::is_payment (selector + exact ABI length).
5. Implement quub-node --dev --http on 8545, chain id 8091, EthereumNode + our executor. No op-node. No Simplex.
6. Add the Sprint 1 unit tests named in SPRINT.md.
7. Add scripts/devnet.sh that starts the node and cast-calls F202.

Stop when cargo test --workspace passes and document how to run the cast call.
If v2.5.2 is missing, use the latest v2.5.* tag and record it.
If a locked decision blocks a choice, quote it. Do not invent a tx type, token, or consensus.
```

When that is green, the next human message is: `Execute Sprint 1.5 from SPRINT.md.`
