# Quub — Sprint 1.5
**After:** Sprint 1 green (`quub-node --dev`, F201–F203 in the map, classifier unit tests).  
**Not this sprint:** `op-node`, Simplex, native token, CCTP, CCIP, new tx type, `quub_` RPC, proxies.

Goal: a real payment on the local node.

```
cast send PaymentToken transferWithMemo(...)
  → F201 allows
  → F202 commits memoHash
  → F212 stores packHash
  → balances move
  → receipt on chain 8091
```

---

## Outcome

| Done when | Proof |
|---|---|
| F210–F213 exist at genesis (or via a one-shot deploy script against `--dev`) | `cast code 0x…F210` non-empty |
| F201 reads freezes from **on-chain PolicyAdmin**, not only in-memory | freeze sender on F211 → next `transferWithMemo` reverts, balances unchanged |
| One `transferWithMemo` is mined | `cast receipt` status 1, `MemoAnchored` + `EvidenceAnchored` logs |
| `scripts/devnet.sh` does start → wait RPC → deploy-or-alloc → transfer → receipt | README documents the exact commands |
| `cargo test --workspace` and `forge test` still green | plus new tests listed below |

---

## Work, in order

### 1. Genesis / deploy F210–F213

Prefer a Foundry script against `http://127.0.0.1:8545` if `--dev` genesis alloc is painful. Either is fine. Addresses stay frozen:

```
F210 PaymentToken
F211 PolicyAdmin
F212 EvidenceAnchor
F213 PaymasterEntry
```

- PaymentToken: mint a test balance to a documented anvil/dev key. Not USDC. Name it `FEE_TOKEN_DEVNET` / payment token. No ticker.
- PolicyAdmin: owner = that same key for `--dev` only. Document that production is dual-control + 24h timelock (do not implement the proxy this sprint).
- EvidenceAnchor: empty.
- PaymasterEntry: listed fee token = the PaymentToken (or the primitives dummy). `quote` works; `takeFee` may still be a stub if debit-from-payer is not wired.

If you allocate in genesis, put the alloc spec in `crates/quub-node` (or a `genesis/` JSON) and record it in README. Do not change chain id `8091`.

### 2. F201 reads PolicyAdmin storage

Sprint 1 F201 used an in-memory `PolicyState`. That is no longer enough.

- `quub-precompiles` stays **pure** where it can: decode calldata, call `quub_policy::check` on a `PolicyState` **passed in**.
- `quub-evm` (the only crate that may touch REVM DB) loads freeze / allow / threshold bits from F211 storage into that `PolicyState` before `run`.
- If storage layout is not worth parsing this sprint: F201 `check` may `CALL` PolicyAdmin. Either path is acceptable. Pick one, test it, do not do both.

Test: `policy_freeze_on_f211_blocks_transfer`.

### 3. Token calls Memo + Anchor

`PaymentToken.transferWithMemo` (already in `contracts/`):

1. call F201 `check` — revert on deny  
2. move balances  
3. call F202 `validateAndCommit` — emit / store memoHash  
4. optional: call F212 with `packHash` if the test supplies one  

If the current Solidity does not call the precompiles, wire those `CALL`s now. Do not invent a new token ABI.

Classifier lengths in `quub-pool` must match this ABI exactly (read the contract; do not hardcode 260).

### 4. One mined happy path

`scripts/pay.sh` (or a section of `devnet.sh`):

```
# after node is up and contracts exist
cast send $F210 "transferWithMemo(...)" --rpc-url http://127.0.0.1:8545 --private-key $DEV
cast receipt $TX
cast call $F202 "..."   # memoHash readable
```

Document the exact calldata in README. Use the same ISO identity set as Sprint 0 `usd_pacs008_stable_hash` so hashes match `quub-iso`.

### 5. Tests to add (names)

Rust:

- `policy_reads_admin_freeze` (or equivalent if the load path is in `quub-evm`)
- `transfer_with_memo_len_matches_solidity` (if classifier constants change)

Foundry (now allowed to touch tests — this is the first sprint that should):

- `test_transferWithMemo_on_forked_devnet` **or** keep unit tests and add `script/` that is run by `devnet.sh`
- `test_frozenSender_stillReverts` against F211, not only the in-memory Solidity mock

Do not delete Sprint 0/1 tests.

---

## Stop conditions

- Do **not** start `op-node`, `reth-optimism-*`, or Simplex.
- Do **not** add a native token, ticker, or genesis ETH productization beyond `--dev` miner ETH.
- Do **not** implement 70/30 payload fill (Sprint 3).
- Do **not** implement CCTP/CCIP adapters (Gateway, later).
- If `services/xzero-*` still exist, rename to `quub-*` in a **separate commit** after 1.5 is green — or first, if it is a one-hour grep. Do not mix a rename failure with genesis work.

---

## After 1.5 is green → Sprint 2

Mode A: swap `EthereumNode` types for OP Stack types, run `op-node` against the same `QuubEvmFactory`, one deposit tx, one L2 block. Same precompiles. Same contracts. That is the next consensus, not Simplex.

---

## Cursor first message

```
Read AGENTS.md, SPRINT-1.5.md, and contracts/src/*.sol.

Sprint 1 is done. Do Sprint 1.5 only.

1. Put PaymentToken, PolicyAdmin, EvidenceAnchor, PaymasterEntry at F210–F213 on the --dev node (genesis alloc or forge script).
2. Make F201 honor PolicyAdmin freezes from chain state.
3. Make transferWithMemo call F201 then F202 (and F212 if a packHash is provided).
4. scripts/devnet.sh (or pay.sh): start node, wait for 8091, send one transferWithMemo, print receipt.
5. cargo test --workspace && forge test must stay green.

No op-node. No Simplex. No token. No new chain id.
Stop and print: contract addresses, tx hash, test counts.
```
