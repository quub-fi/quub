Read AGENTS.md, ARCHITECTURE.md, SPRINT-1.5.md, scripts/devnet.sh, contracts/src/*.sol, crates/quub-evm/src/precompiles.rs, crates/quub-evm/src/wrap.rs, services/quub-iso (or services/xzero-iso) src.

Do not change production code. Do not start op-node. Do not rename crates. This is validate-only.

Write ONE file at the repo root:

    artifacts/sprint15-validate.txt

Create artifacts/ if needed. Append as you go. If a command fails, write the command, exit code, and stderr, then continue. Do not stop the file early.

Use RPC http://127.0.0.1:8545
ANVIL0=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
ANVIL0_PK=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
F201=0x000000000000000000000000000000000000F201
F202=0x000000000000000000000000000000000000F202
F203=0x000000000000000000000000000000000000F203
F210=0x000000000000000000000000000000000000F210
F211=0x000000000000000000000000000000000000F211
F212=0x000000000000000000000000000000000000F212
F213=0x000000000000000000000000000000000000F213
CLAIMED_TX=0xa3cf19db17b61eec6063de90e3af8034d2fa1f1e468de3a784d8c31c1d880638

If the node is not up, start it the same way scripts/devnet.sh starts quub-node, wait until cast chain-id is 8091, then continue. Leave the node running.

Identity set for hash checks — read the exact values from scripts/devnet.sh. If missing, use the Sprint 0 set and WRITE which you used:
  endToEndId, uetr, instrId, ccy, msgType, packHash, recipient TO

File format — exact section headers:

------------------------------------------------------------------------
SECTION 0 META
------------------------------------------------------------------------
date (UTC)
git rev-parse HEAD
git status -sb
uname -a
which cast forge cargo quub-node
cast --version
forge --version
rustc --version

------------------------------------------------------------------------
SECTION 1 NODE
------------------------------------------------------------------------
cast chain-id --rpc-url http://127.0.0.1:8545
cast client --rpc-url http://127.0.0.1:8545
cast block-number --rpc-url http://127.0.0.1:8545
ps aux | grep -E 'quub-node|anvil|op-node' | grep -v grep

------------------------------------------------------------------------
SECTION 2 CODE AND STATE
------------------------------------------------------------------------
For each of F201 F202 F203 F210 F211 F212 F213:
  address
  cast codesize <addr>
  first 20 chars of cast code <addr> (or "empty")

cast call F210 "name()(string)"
cast call F210 "symbol()(string)"
cast call F210 "decimals()(uint8)"
cast call F210 "totalSupply()(uint256)"
cast call F210 "balanceOf(address)(uint256)" $ANVIL0
cast call F211 "ownerA()(address)"
cast call F211 "ownerB()(address)"
cast call F211 "paused()(bool)"
cast call F211 "threshold()(uint256)"
cast call F211 "feeToken()(address)"
cast call F211 "frozen(address)(bool)" $ANVIL0

------------------------------------------------------------------------
SECTION 3 STORAGE LAYOUT
------------------------------------------------------------------------
cd contracts
forge inspect PolicyAdmin storageLayout
forge inspect PaymentToken storageLayout
forge inspect EvidenceAnchor storageLayout
forge inspect PaymasterEntry storageLayout

If crates/quub-node/alloc/*.json exists, cat those files.

------------------------------------------------------------------------
SECTION 4 PRECOMPILE REGISTRATION
------------------------------------------------------------------------
rg -n "new_stateful|DynPrecompile::new\(|spec >=|spec ==" crates/quub-evm/src
echo "----- wrap.rs -----"
cat crates/quub-evm/src/wrap.rs
echo "----- precompiles.rs -----"
cat crates/quub-evm/src/precompiles.rs

------------------------------------------------------------------------
SECTION 5 CLAIMED RECEIPT
------------------------------------------------------------------------
cast receipt $CLAIMED_TX --rpc-url http://127.0.0.1:8545 --json
If that fails, write FAILED and why.

cast tx $CLAIMED_TX --rpc-url http://127.0.0.1:8545 --json
Write input byte length (hex chars/2 - 1 for 0x).

------------------------------------------------------------------------
SECTION 6 FRESH HAPPY PATH
------------------------------------------------------------------------
Record balances of ANVIL0 and TO before.
Run the exact transferWithMemo from scripts/devnet.sh (include packHash).
Write the new tx hash.
cast receipt <newtx> --json
cast receipt <newtx>  (human)
Write log topic0 values.
cast sig-event for MemoAnchored and EvidenceAnchored as defined in the contracts (read the Solidity, do not guess the signature).

------------------------------------------------------------------------
SECTION 7 HASH RECOMPUTE
------------------------------------------------------------------------
Print the exact abi.encode argument list used.
Recompute:
  cast abi-encode "f(bytes32,bytes16,bytes32,bytes3,uint8,address)" <e2e> <uetr> <instr> <ccy> <msgType> $ANVIL0
  cast keccak <that payload>
Write computed hash.
Write memoHash from the new receipt.
Write:
  cast call F202 validateAndCommit(...) --from $ANVIL0
  cast call F202 validateAndCommit(...) --from 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
(second key = anvil account 1). These two hashes MUST differ. Write both.

Write:
  cast call F201 "check(address,address,address,uint256,bytes32)(bool,uint16)" F210 $ANVIL0 $TO 1 0x0 --from $ANVIL0
Expect revert / empty. Write the result.

------------------------------------------------------------------------
SECTION 8 FREEZE
------------------------------------------------------------------------
BEFORE_FROM=$(cast call F210 balanceOf ANVIL0)
BEFORE_TO=$(cast call F210 balanceOf TO)

cast send F211 "freeze(address)" $ANVIL0 --private-key $ANVIL0_PK
cast call F211 "frozen(address)(bool)" $ANVIL0

Attempt the SAME transferWithMemo again.
Write: success or revert data / reason.

AFTER_FROM AFTER_TO balances.
Expect: revert, balances equal to BEFORE_*.

cast send F211 "unfreeze(address)" $ANVIL0 --private-key $ANVIL0_PK
Attempt transferWithMemo a third time.
Write: success tx hash or revert.

------------------------------------------------------------------------
SECTION 9 TESTS
------------------------------------------------------------------------
cargo test --workspace
cd contracts && forge test
Write the summary lines only plus any FAILED test names.

------------------------------------------------------------------------
SECTION 10 AGENT CONCLUSION
------------------------------------------------------------------------
For each, write PASS/FAIL/UNKNOWN and one line why:
- chain id 8091 and client is quub-node not anvil
- F210-F213 code non-empty
- claimed tx still on this node
- fresh tx status 1 with both events
- recomputed hash matches receipt (origin = ANVIL0)
- F202 hash changes when --from changes
- F201 direct call from EOA reverts
- freeze then transfer reverts and balances unchanged
- unfreeze then transfer succeeds
- F201 and F202 registered with new_stateful
- cargo + forge still green
- no op-node running

Stop. Do not implement Sprint 2. Print the path artifacts/sprint15-validate.txt and its byte size.
