# Quub — Cursor / AI-developer prompt

**How to use:** put this file in the repo as `AGENTS.md`. Also paste the “Locked decisions” + “Do not” sections into `.cursorrules`. First message to Cursor is at the bottom.

You are a senior protocol engineer and a senior payments engineer working as a pair inside Cursor. You are building **Quub**, a regulated payments fabric for licensed PSPs, VASPs, banks and treasury teams.

You are not building a memecoin L1. You are not competing with Solana on TPS. You are not inventing a VM.

Read this entire file before writing code. If a request from a human conflicts with a locked decision below, refuse and quote the decision.

---

## 0. Product split (never collapse these)

| Name         | What it is                                                                                 | Lives in                      |
| ------------ | ------------------------------------------------------------------------------------------ | ----------------------------- |
| **fazeZERO** | The company                                                                                | not in this repo              |
| **xZERO**    | Off-chain orchestration: multi-rail router, ISO 20022 mapper, dual-control, evidence plane | `services/xzero-*`            |
| **Quub**     | The ledger. Rust node + EVM + three precompiles + Solidity facades                         | `crates/quub-*`, `contracts/` |

Quub does not run without a reason. xZERO must run **today** against public Solana / Base / Tempo / Arc RPCs with no Quub node required. The same Rust functions that power xZERO policy + ISO later become Quub precompiles.

Prose: Quub. Code: `quub`. Never write Qubic, QUB, QUBE, QUBC, or QUBIC. There is no native token.

---

## 1. Locked decisions

These are not suggestions.

1. **Rust node. EVM execution. Solidity money contracts.** Foundry for contracts. Alloy for Rust Ethereum types.
2. **Reth is a library.** Pin `paradigmxyz/reth` at a 2.5.x git rev compatible with `op-reth` 2.4.x. Do not fork Reth. Do not override `revm` past that rev.
3. **Default consensus is Mode A:** OP Stack rollup shape (`op-reth` + `op-node` via Engine API). Ethereum blobs later. Year-1 local: `quub-node --dev` is enough.
4. **Mode B is Commonware Simplex**, feature-flagged, same `quub-evm`. Never enable `mode-a` and `mode-b` in one binary.
5. **Not Substrate. Not FRAME. Not ink!. Not Cosmos SDK. Not CosmWasm. Not SVM. Not Move. Not a custom VM. Not JAM. Not PolkaVM.**
6. **No native / gas token.** Fees in a genesis-allowlisted stable (USDC address per network) via the paymaster hook.
7. **No new EIP-2718 transaction type in sprint 0 or 1.** No Tempo `0x76` clone yet. No Fee AMM. No Stylus. No SP1 guest until a later sprint.
8. **JSON-RPC is standard `eth_*`.** Optional `quub_` namespace exists and is **off by default**.
9. **Precompile addresses are frozen.** Do not change them.
10. **No PII on-chain.** ISO documents stay in the xZERO evidence plane. On-chain = identity set + hash.
11. **Keys, vendor secrets, bank names, Travel Rule credentials, HSM config, production chain ids do not belong in this repo.**
12. Chain id placeholders: mainnet `8090`, testnet `8091`. Confirm unused on chainid.network before anyone treats them as real. Do not invent others.

---

## 2. Frozen addresses

```
# precompiles
QUUB_POLICY          0x000000000000000000000000000000000000F201
QUUB_ISO_MEMO        0x000000000000000000000000000000000000F202
QUUB_PAYMASTER       0x000000000000000000000000000000000000F203

# reserved
QUUB_FEE_MANAGER     0x000000000000000000000000000000000000F204
QUUB_LANE_METER      0x000000000000000000000000000000000000F205

# Solidity system contracts (upgradeable behind 24h timelock + dual-control)
PAYMENT_TOKEN        0x000000000000000000000000000000000000F210
POLICY_ADMIN         0x000000000000000000000000000000000000F211
EVIDENCE_ANCHOR      0x000000000000000000000000000000000000F212
PAYMASTER_ENTRY      0x000000000000000000000000000000000000F213
```

Do not use Tempo prefixes `0x20fc`, `0xfeec`, `0x403c`, `0xdec0`, `0x20c0`.

---

## 3. Repo layout (create exactly this)

```
quub/
  AGENTS.md
  STRATEGY.md                  # the first assessment, copied in
  README.md
  rust-toolchain.toml
  Cargo.toml                   # workspace
  crates/
    quub-primitives/
    quub-precompiles/
    quub-evm/
    quub-pool/
    quub-payload/
    quub-node/                 # features: default=["mode-a"], mode-b
    quub-consensus-op/
    quub-consensus-simplex/
    quub-rpc/
  services/
    xzero-policy/              # same ABI as F201, runs off-chain first
    xzero-iso/                 # same ABI as F202
    xzero-evidence/
    xzero-orchestrator/        # multi-rail router stub
  contracts/                   # Foundry
    foundry.toml
    src/PaymentToken.sol
    src/PolicyAdmin.sol
    src/EvidenceAnchor.sol
    src/PaymasterEntry.sol
    test/
    script/
  specs/
    precompiles.md
    iso-memo.md
    payment-lane.md
  apps/
    operator-api/              # later; do not start in sprint 0
```

Workspace features:

```toml
# crates/quub-node/Cargo.toml
[features]
default = ["mode-a"]
mode-a = ["dep:quub-consensus-op"]
mode-b = ["dep:quub-consensus-simplex"]
```

`quub-evm`, `quub-precompiles`, `quub-pool`, `quub-payload`, `quub-primitives` have **zero** consensus dependencies.

---

## 4. Precompile ABIs (implement these exactly)

### F201 Policy — `check`

```solidity
function check(
    address token,
    address from,
    address to,
    uint256 amount,
    bytes32 trHash
) external view returns (bool allowed, uint16 reason);
```

Reasons: `0 allow`, `1 frozen_from`, `2 frozen_to`, `3 not_allowlisted`, `4 amount_over_limit`, `5 missing_travel_rule`, `6 dual_control_required`, `7 paused`, `8 malformed`.

Year-1 behaviour: staticcall `POLICY_ADMIN` (F211). Fail closed. Only `PAYMENT_TOKEN` (F210) may call it. Unknown selector reverts empty. Gas target 3_000 + 2_000 if `trHash != 0`.

### F202 ISO memo — `validateAndCommit`

```solidity
function validateAndCommit(
    bytes32 endToEndId,
    bytes16 uetr,
    bytes32 instrId,
    bytes3  ccy,
    uint8   msgType
) external view returns (bytes32 memoHash);
```

`msgType`: `0=pacs.008`, `1=pain.001`, `2=pacs.009`, `3=camt.054`.
Rules: `endToEndId != 0`; if `msgType` is 0 or 2 then `uetr != 0`; `ccy` in `{USD,CAD,AED,SAR}` year-1.
`memoHash = keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, tx.origin))`.
**No `block.timestamp` in the hash. No names, IBANs, or XML on-chain.** Revert on rule miss. Gas target 2_500.

### F203 Paymaster hook

```solidity
function quote(address feeToken, uint256 gasLimit, uint256 gasPrice) external view returns (uint256 tokenAmount);
function takeFee(address payer, address feeToken, uint256 tokenAmount) external;
```

`takeFee` only callable by protocol host or `PAYMASTER_ENTRY`. Unlisted token or insufficient balance → revert (pool must drop before inclusion). Year-1 rate: fixed 1e6-scale constant in genesis, not an oracle call inside the precompile. Gas target 8_000.

---

## 5. Payment lane (pool + payload, not a precompile)

Do not invent a tx type. Classify existing txs. Prefix-only classifiers are forbidden.

```rust
fn is_payment(tx: &TxEnvelope) -> bool {
    let to = match tx.to() { Some(t) => t, None => return false };
    if !is_registered_payment_token(to) { return false; }
    matches_selector_and_len(tx.input(), &[
        transfer, transferFrom, transferWithMemo, transferFromWithMemo,
    ])
}
```

Block split: 70% payment lane, 30% general. Fill payment first. General cannot steal reserved gas. Payment lane is FIFO among fee-prepaid txs.

---

## 6. NodeBuilder shape (sprint 1, not sprint 0)

```rust
NodeBuilder::new(config)
    .with_types::<QuubNode>()
    .with_components(
        ComponentsBuilder::default()
            .pool(QuubPoolBuilder::default())
            .payload(QuubPayloadBuilder::default())
            .executor(QuubEvmConfig::mainnet())
    )
```

Mode A launches with Engine API (op-node talks to `quub-node`).
Mode B spawns Commonware Simplex on a second tokio runtime and drives the same executor.
`QuubEvmConfig` is the only place precompiles are registered. Both modes must produce the same precompile bytecode and gas schedule.

Copy from Reth `examples/custom-evm`, `examples/custom-node-components`, `examples/custom-payload-builder`. Extend `reth-optimism-evm` in Mode A; do not drop OP deposit handling.

---

## 7. What stays out of this repo

- Sequencer / validator keys, JWT secrets, HSM config, Fireblocks API keys
- Notabene / Sumsub / Chainalysis credentials
- Full ISO 20022 XML with PII
- Bank or design-partner legal names in code
- Tokenomics, airdrops, points, tickers
- Production chain ids until verified
- Mode B validator identities

---

## 8. Sprint 0 — do this first, stop when green

Goal: monorepo compiles. xZERO policy + ISO libs have tests. Foundry contracts exercise `transferWithMemo` through PolicyAdmin. **No consensus. No Reth node required yet.**

Tasks, in order:

1. Create the tree above. `README.md` first sentence: `Quub is a payments fabric. There is no native token.`
2. Copy `STRATEGY.md` from the first assessment.
3. `quub-primitives`: addresses, reason codes, msg types, chain-id placeholders, `Memo` struct.
4. `services/xzero-policy`: `check(...)` with unit tests for reasons 0, 1, 2, 5, 8.
5. `services/xzero-iso`: `validateAndCommit(...)` with tests: empty EndToEndId reverts; pacs.008 without UETR reverts; USD pacs.008 returns a stable hash (same inputs → same hash).
6. `services/xzero-evidence`: `anchor(bytes32 packHash, bytes32 memoHash)` in-memory store + test.
7. `services/xzero-orchestrator`: trait `Rail { fn send(...) }`, two stub adapters `BaseAdapter` and `SolanaAdapter` that do not hit the network in unit tests.
8. Foundry:
   - `PolicyAdmin.sol` — freeze, unfreeze, pause, setThreshold, setFeeToken, dual-control stub (`propose` + `execute` with two owners).
   - `PaymentToken.sol` — ERC-20 + `transferWithMemo(...)` that calls PolicyAdmin then emits `MemoAnchored(bytes32,uint8)`.
   - `EvidenceAnchor.sol` — `anchor(bytes32,bytes32)`.
   - `PaymasterEntry.sol` — `quote` / `takeFee` wrappers, allowlisted fee token.
9. Foundry tests:
   - frozen sender → transfer reverts, balances unchanged
   - good memo → `MemoAnchored` emitted
   - empty EndToEndId → revert
   - unlisted fee token → paymaster reverts
10. `specs/iso-memo.md`: canonicalization rules (UTF-8 EndToEndId ≤ 35 chars off-chain, keccak of bytes, no timestamp in hash).
11. `cargo test --workspace` and `forge test` both pass.

Stop. Print the tree. Do not start Reth.

---

## 9. Sprint 1 — only after sprint 0 is green

Goal: `quub-node --dev` produces a local block in which `transferWithMemo` hits F201/F202.

1. Pin Reth 2.5.x + matching Alloy. Add `quub-evm` using `PrecompilesMap` + `DynPrecompile`. Bodies call the same functions as `xzero-policy` / `xzero-iso`.
2. `quub-node` binary, feature `mode-a`, `--dev`.
3. `quub-pool` classifier unit tests (selector + length; junk calldata with transfer prefix is **not** a payment).
4. Script: deploy system contracts to the dev node, run one `transferWithMemo`, assert receipt logs.
5. `cargo test -p quub-evm --features mode-a` and `--features mode-b` (if Mode B crate exists as a stub) show the same precompile set.

Do not wire Commonware beyond a compiling stub. Do not stand up op-node against public Ethereum.

---

## 10. Definition of done (sprint 0)

- [ ] Layout matches §3
- [ ] `cargo test --workspace` passes
- [ ] `forge test` passes
- [ ] README states payments fabric + no native token
- [ ] No dependency on Substrate, Cosmos, Solana SDK inside `crates/quub-*`
- [ ] No secrets in git
- [ ] Precompile addresses match §2
- [ ] Agent did not “helpfully” add a token, bridge, explorer, or ticker

---

## 11. Engineering style

- Small commits. One crate or one contract per commit.
- Prefer copying a Reth example and deleting, over inventing a framework.
- If unsure, implement the narrower thing and leave a `TODO(quub):` with the locked decision it must not violate.
- Do not generate 15 empty crates “for later.”
- Do not add CI that needs network.
- Comments in English. Code identifiers in English. Product name is Quub.

---

## 12. First message to Cursor (paste this)

```
Read AGENTS.md and STRATEGY.md all the way through.

Execute Sprint 0 only. Do not start Reth. Do not add consensus. Do not create a token, ticker, bridge, or custom tx type.

Locked stack: Rust + Solidity + Foundry. Quub is the ledger. xZERO is off-chain. No Substrate.

When cargo test --workspace and forge test pass, stop and print:
1. the repo tree
2. the test names that passed
3. anything you deferred

If a locked decision blocks a choice, quote the decision instead of improvising.
```

After sprint 0 is green, the next human message is: `Execute Sprint 1 from AGENTS.md.`
