Read AGENTS.md, ARCHITECTURE.md, SPRINT-2.md, crates/quub-consensus-op/CONFLICT.md, crates/quub-evm/src/{factory.rs,precompiles.rs,wrap.rs}, crates/quub-node/src/main.rs, scripts/devnet.sh.

Sprint 1.5 is accepted. Overflow is committed. Do not re-litigate F201–F213, the memo hash, or chain id 8091.

You are finishing Sprint 2. The first Mode A attempt correctly STOPPED: paradigmxyz/reth v2.5.2 has no optimism crates. The human chose option 2 (ADR-016). That pin change IS Sprint 2. Do not open Sprint 3. Do not start Simplex. Do not add a token.

## ADR-016 pins (exact)

One Reth remote. If Cargo.lock contains both paradigmxyz/reth and op-rs/reth, STOP.

```
reth-* crates:
  git = "https://github.com/op-rs/reth"
  rev = "aef8d3ef92117f91455e16969f0adf5bf7c6e9e1"

OP crates (reth-optimism-node, reth-optimism-evm, primitives, …):
  git = "https://github.com/ethereum-optimism/optimism"
  tag = "op-reth/v2.4.4"

op-node binary (scripts / README, not Cargo):
  v1.19.7   (minimum v1.19.1)
```

That rev is what op-reth/v2.4.4 itself uses. Do not pick a newer op-rs/reth commit. Do not float main.

## Work order

### A. Record the decision
- Add ADR-016 to the repo (execution pin table above).
- Rewrite CONFLICT.md as resolved by ADR-016. Keep the old facts as history.
- Update AGENTS.md and ARCHITECTURE.md: Reth source is now op-rs/reth @ aef8d3ef + optimism tag op-reth/v2.4.4. Pair op-node v1.19.7.

### B. Pin swap
- Remove paradigmxyz/reth tag v2.5.2 from workspace Cargo.toml.
- Point every reth / reth-ethereum dependency at op-rs/reth rev aef8d3ef92117f91455e16969f0adf5bf7c6e9e1.
- Add the reth-optimism-* crates you need from ethereum-optimism/optimism tag op-reth/v2.4.4 (crates live under rust/ in that repo; use the package names Cargo expects).
- cargo check -p quub-evm -p quub-node
- If check fails after deps are downloaded, write artifacts/pin-swap-fail.txt with the rustc error and STOP. Do not implement Engine API on a red build.

### C. Port --dev onto the new pin
- Adapt QuubEvmFactory, wrap.rs, precompile inject, QuubExecutorBuilder, quub-node --dev to the new types.
- Keep:
  - F201 F202 F203 addresses
  - F201/F202 DynPrecompile::new_stateful
  - inject when spec >= PRAGUE (or the OP-equivalent fork name if the new tree renamed it — same intent)
  - F201 sload F211; caller must be F210
  - F202 hash = keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, tx.origin))
  - genesis alloc F210–F213, chain id 8091, HTTP 8545
- scripts/devnet.sh must still mine transferWithMemo status 1.
- cargo test --workspace && forge test must stay green.

### D. Mode A launch (only if C is green)
- quub-node --engine: HTTP 9545, authrpc 9551, JWT file. Optimism node types + same Quub precompiles. Never compile Simplex into this binary.
- scripts/mode-a.sh:
  1. local L1 (anvil or documented geth)
  2. op-node v1.19.7
  3. quub-node --engine
  4. wait until cast chain-id on 9545 is 8091
  5. one L1 deposit if the L2 payer needs gas
  6. cast send F210 transferWithMemo on 9545
  7. print L2 receipt
- If op-node is stopped, L2 must not keep mining (no leftover --dev miner on that process).
- Same freeze story on L2: freeze F211 → send fails → unfreeze → send works.

## Forbidden
- Simplex / mode-b enabled
- Native token / ticker
- New chain id
- 70/30 lane
- xzero rename mixed into this branch
- Sepolia / public RPC / Superchain registry
- A second Reth git
- Bumping off the rev/tag above to “make it compile”

## Stop and print
1. Cargo.toml reth + optimism dependency lines
2. cargo check result
3. --dev: chain-id, F213 codesize, transferWithMemo tx hash (or pin-swap-fail.txt path)
4. --engine: op-node version, L2 chain-id, deposit hash, L2 payment hash, freeze result
   or “Mode A not started because pin-swap failed”
5. cargo test + forge test counts
