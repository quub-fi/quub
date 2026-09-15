# Quub — Sprint 2
**After:** Sprint 1.5 accepted on `--dev` (payment + freeze + origin-in-hash).  
**This sprint:** close 1.5 leftovers, then run the **same** execution crate under OP Stack (`op-node` + Engine API). One L1 deposit. One L2 block that contains a `transferWithMemo`.

Not this sprint: Simplex, native token, new chain id, 70/30 lane, `quub_` RPC, proxies / 24h timelock, CCTP/CCIP, `xzero-*` rename (still its own commit).

---

## Overflow from 1.5 (do first, one PR)

These are hygiene. Do not start `op-node` until they are in.

1. **Commit the dirty tree.** Sprint 1.5 lives in untracked / modified files (`policy_load.rs`, `slots.rs`, `alloc/`, `alloc_genesis.rs`, wrap/precompiles/PaymentToken). `ARCHITECTURE.md` and `SPRINT-1.5.md` belong in the repo. Do not commit anvil keys as if they were production.
2. **F201 EOA gate, proven.** The validate file failed on `cast` (`0x0` is not a `bytes32`). Retry:
   ```
   cast call $F201 "check(address,address,address,uint256,bytes32)(bool,uint16)" \
     $F210 $ANVIL0 $TO 1 \
     0x0000000000000000000000000000000000000000000000000000000000000000 \
     --from $ANVIL0
   ```
   Expect revert / empty. A call that only works when `msg.sender` is F210 is the rule. Add a Foundry or Rust test so this does not depend on a live node.
3. **F213 smoke.** `cast code $F213` non-empty. `cast call` the allowlist / `quote` path if the ABI exists. `takeFee` may still stub. Do not implement stateful debit.
4. **`--dev` datadir.** `devnet.sh` should print the data directory. Optional: persist it so receipts survive a restart. Do not treat a wiped `--dev` hash as mainnet history.
5. **`quub-node` on PATH for engineers.** README: `cargo build -p quub-node && export PATH=./target/debug:$PATH`.

Stop overflow when: `git status` is clean enough to branch `sprint-2`, F201 EOA test exists, F213 has code.

---

## Sprint 2 outcome

```
L1 (local anvil / geth, chain id 900 or whatever OP local-dev uses)
   deposit ETH or a test ERC-20 *for gas on L1 only*
        │
        ▼
op-node  ──Engine API──►  quub-node (same QuubEvmFactory, F201–F203)
        │
        ▼
L2 block on chain id 8091
   transferWithMemo on F210
   receipt status 1
   MemoAnchored + EvidenceAnchored
```

Done when `scripts/mode-a.sh` prints: L1 deposit tx, L2 block number ≥ 1, L2 payment tx hash, both events.

---

## Locked Mode A choices

| Item | Choice |
|---|---|
| Execution | Existing `QuubEvmFactory` / `QuubExecutorBuilder`. Do not fork Reth. |
| Consensus | `op-node` (Optimism official binary or the version OP documents for the Reth pin). Quub does not reimplement derivation. |
| L2 chain id | **8091** (same as `--dev`). Do not invent 8092. |
| L1 | Local only. `anvil` or `op-geth` in `scripts/mode-a.sh`. Not Sepolia this sprint. |
| DA | Whatever the local OP devnet uses (calldata is fine). Blobs on public L1 are later. |
| Sequencer | One local key in the script. Not production. |
| Genesis on L2 | Same F210–F213 alloc as Sprint 1.5. Same slots. |
| RPC | `eth_*` only. L2 HTTP `127.0.0.1:9545` (keep 8545 for `--dev` so both can exist). |
| Pin | Record in README: Reth tag, `op-node` version, L1 image. |

`--dev` stays. Mode A is a **second** launch path: `quub-node --dev` vs `quub-node --engine` (name the flag in README). Same binary, different node types.

---

## Work order

### 2.1 `quub-consensus-op` (replace DEFERRED.md)

Thin crate: OP block / payload types and the Engine API surface `quub-node` needs. Prefer `reth-optimism` / `op-reth` crates **at a version that matches the Reth 2.5.2 line**. If the crate matrix will not compile on 2.5.2, **stop and write the version conflict in README** — do not silently bump Reth.

Do not copy all of `op-reth`. Wire:

- `EngineApi` on the node
- OP deposit tx type accepted by the executor
- JWT secret file for `op-node`

### 2.2 `quub-node` Engine mode

`quub-node --engine --http 9545 --authrpc 9551 --jwt jwt.txt`

Uses OP `Node` types + `QuubExecutorBuilder`, not `EthereumNode`.  
`--dev` path must still compile and pass `scripts/devnet.sh`.

### 2.3 Local OP stack script

`scripts/mode-a.sh`:

1. Start L1 (anvil or documented op-geth).
2. Start `quub-node --engine`.
3. Start `op-node` pointed at both, with the documented genesis / rollup.json.
4. Wait until `cast block-number --rpc-url http://127.0.0.1:9545` ≥ 1.
5. If F210–F213 are missing on this genesis, fail. Do not `CREATE` onto `0x…F210`.
6. `cast send` F210 `transferWithMemo` on **9545** (same identity set as 1.5).
7. Print L2 tx hash + receipt.

A deposit that creates an L2 account with L1 ETH-for-gas is required if the L2 payer has no balance. Document the deposit command. The payment token is still F210, not L1 ETH.

### 2.4 Tests

Keep every Sprint 0 / 1 / 1.5 test green.

Add:

- `mode_a_factory_builds` — `QuubEvmFactory` still registers F201–F203 on `spec >= PRAGUE`.
- One integration test **or** the script above. Prefer the script; do not add a flaky CI that needs Docker if Docker is not already in the repo.

### 2.5 README

Two commands:

- Laptop payment: `scripts/devnet.sh`
- Mode A: `scripts/mode-a.sh`

Pins, ports (8545 vs 9545), jwt path, “this is not Sepolia.”

---

## Stop conditions

- Do **not** start Commonware / Simplex / `mode-b`.
- Do **not** add a ticker, genesis QUUB, or change F210 into gas.
- Do **not** change chain id 8091.
- Do **not** implement 70/30 payload fill (Sprint 3).
- Do **not** mix the `xzero-*` rename into this branch.
- Do **not** bump Reth off 2.5.x to make `op-reth` compile without writing the conflict down and waiting.
- Do **not** register with Superchain or publish `rpc.quub.network`.

---

## Acceptance

- Overflow PR merged or on the same branch, F201 EOA test green.
- `cargo test --workspace` and `forge test` still green.
- `scripts/devnet.sh` still mines a `--dev` payment (8545).
- `scripts/mode-a.sh` prints an L2 `transferWithMemo` receipt status 1 with `MemoAnchored`.
- Freeze on F211 still reverts a second L2 send (same 1.5 story, now through `op-node`).
- `ps` during Mode A: `quub-node` + `op-node`. No Simplex process.

Print: Reth pin, op-node version, L2 tx hash, test counts.

---

## Cursor first message

```
Read AGENTS.md, ARCHITECTURE.md, SPRINT-2.md.

Sprint 1.5 is accepted. Do Sprint 2.

Part A — 1.5 overflow, first commit:
1. Commit policy_load.rs, slots.rs, alloc/, genesis, wrap/precompile/token changes. Do not commit secrets.
2. Add a test that F201 check from a non-F210 caller reverts. Use a real bytes32 zero, not 0x0.
3. Prove F213 has code on --dev (cast codesize) or document why genesis skipped it.
4. README: how to put target/debug on PATH.

Part B — Mode A:
1. quub-consensus-op: Engine API + OP deposit types against the current Reth 2.5.x pin. If op-reth will not compile on that pin, stop and write the conflict; do not bump Reth.
2. quub-node --engine on 9545 / authrpc 9551. Keep --dev on 8545 working.
3. scripts/mode-a.sh: local L1 + op-node + quub-node, wait for L2 block, transferWithMemo on F210, print receipt.
4. Same F210–F213 alloc. Same memo hash. No new chain id.

No Simplex. No token. No xzero rename. No 70/30 lane.

Stop and print: overflow commit hash, Reth pin, op-node version, L2 tx hash, cargo + forge counts.
```
