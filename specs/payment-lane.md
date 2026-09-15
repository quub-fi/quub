# Payment lane

Classify existing transactions; do **not** invent a new EIP-2718 type (locked decision §7).

Prefix-only classifiers are forbidden. A payment tx must:

1. Target a registered payment token address
2. Match a known selector **and** exact calldata length for
   `transfer` / `transferFrom` / `transferWithMemo` / `transferFromWithMemo`

Block split (Sprint 1+): 70% payment lane, 30% general. Fill payment first. General cannot steal reserved gas. Payment lane is FIFO among fee-prepaid txs.

Implementation crate: `crates/quub-pool` (deferred to Sprint 1).
