Sprint 3 is the payment lane on the node you already have. Same pin. Same 8091. No Simplex. No token.
Locked rule
A tx is payment only if to == F210 and the selector is transferWithMemo (9-arg, locked calldata length). Everything else is general, including plain transfer on F210.
text1. Payments first, up to 70% of block gas 2. General next, up to 30% 3. Leftover: payments, then general
(empty payment lane may fill 100% general — do not ship a hollow 70%)
When both lanes are full, a higher-tip general must not evict a payment from the first 70%. That is the product test.
Deposits stay engine-injected (Mode A). They are not pool-lane traffic.
Where

CrateJobquub-primitivesPAYMENT_LANE_BPS = 7000, selector, calldata lengthquub-poolClassifierquub-payload70/30 fill iteratorquub-nodeSame builder on --dev and --engine
Do not put 70/30 in PolicyAdmin. This is block building.
Acceptance

Unit tests: classifier matrix + four fill cases (mixed, no payments, no general, huge general cannot steal the 70%).
Live --dev (8545): flood memos + dummy txs; one block printed with payment vs general counts; payment gas share ≥ ~65% when payment demand is high (packing slack).
--engine uses the same builder type (not stock). Live 9545 flood if Mode A is up; code-path proof is required either way.
Freeze still reverts. Kill-test from Sprint 2 still holds.
Pins unchanged: aef8d3ef, op-reth/v2.4.4, rustc 1.96.

Read AGENTS.md, ARCHITECTURE.md, ADR-016.

Sprint 2 is accepted. Do Sprint 3 only.

Payment lane: 70% of block gas reserved for transferWithMemo on F210
(selector + address). 30% general. Spill leftover so blocks are not empty.
Lane beats gas price. Empty payment lane may fill with general.

1. quub-pool classifier + tests.
2. quub-payload 70/30 fill + tests (no node required).
3. Wire that builder into quub-node --dev AND --engine.
4. Mixed flood on 8545; print one block’s payment/general counts and gas.
5. Same on 9545 if Mode A is up.
6. ADR-017 in ARCHITECTURE.md. PAYMENT_LANE_BPS = 7000.

Pins stay: op-rs/reth aef8d3ef, op-reth/v2.4.4, rustc 1.96, chain 8091,
F210–F213, origin-in-hash.

No Simplex. No token. No Reth bump. No fee-market redesign.
No 70/30 in PolicyAdmin.

Stop and print: block hash, payment vs general counts, gas per class,
cargo + forge counts.
