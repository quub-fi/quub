# Payment lane (ADR-017 / Sprint 3)

Classify existing transactions; do **not** invent a new EIP-2718 type.

Prefix-only classifiers are forbidden. A payment tx must:

1. Target **F210** (`PAYMENT_TOKEN`)
2. Match `transferWithMemo` selector **and** exact calldata length (`4 + 32 * 9`)

Plain `transfer` / `transferFrom` on F210 are **general**.

Block fill (`PAYMENT_LANE_BPS = 7000`):

1. Payments first, up to 70% of block gas (FIFO among fee-prepaid)
2. General next, up to 30% (tip order)
3. Leftover: payments, then general

Empty payment lane may fill 100% general (no hollow 70%). Lane beats tip: a higher-tip general must not steal reserved payment gas while payments wait.

Deposits (Mode A) stay engine-injected; they are not pool-lane traffic. Do not put 70/30 in PolicyAdmin.

Crates: `quub-pool` (classifier), `quub-payload` (`fill_lanes`).
