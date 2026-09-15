Use cases Quub can actually carry. Split by **what the stack already does**, **what Sprint 6 + ops unlocks**, and **what we should not sell**.

### Works on the node you have (8091)

1. **Identified internal transfer** — `transferWithMemo` with end-to-end id, UETR, instruction id, currency. Memo hash includes `tx.origin`.
2. **Evidence pack** — same payment writes `packHash` to F212 so ops can prove “this file goes with this payment.”
3. **Sanctions freeze** — F211 freeze from either owner, immediate. Frozen sender cannot move F210. Balances unchanged on reject.
4. **Controlled unfreeze** — Owner A proposes, Owner B confirms. One key cannot walk money back alone (`--dev` already two keys).
5. **Halt** — pause via two-key confirm; freeze stays one-key.
6. **Fee on the identified payment only** — F213 takes a flat listed-token fee on memo. Plain `transfer` stays free (general lane).
7. **Priority settlement** — 70% of the block reserved for memos so a wire is not crowded out by junk traffic.
8. **Operator submit without Solidity** — `POST /v1/payments` on `:8080`, idempotent on `endToEndId`.
9. **Operator freeze without Solidity** — `POST /v1/freeze` then next pay is 422 reason 1.
10. **Health / chain guard** — API refuses if RPC is not chain 8091.
11. **Dev replay** — `--dev` one-box demo for banks (anvil keys, QPT dummy dollars).
12. **Derived L2 demo** — Mode A: same contracts, `op-node` + geth L1, kill-test (L2 stops when consensus stops).

### After Sprint 6 + a public testnet

13. **USDC to Base (CCTP)** — Quub records who/why; Circle burns on Sepolia and mints on Base Sepolia. Quub is not the bridge.
14. **Rail retry** — Quub mined, CCTP failed → retry the rail, do not post a second memo.
15. **Policy-gated off-ramp** — freeze on Quub blocks the CCTP burn. Destination chain never sees the dollars.
16. **Treasury sweep to Base** — ops pays on Quub, USDC lands on a Base address they already use with Coinbase/Circle.
17. **Reconciliation file** — one `endToEndId` → Quub tx + memoHash + burn + mint. Core-banking can match pacs.008 to chain.

### Partner / next level (architecture supports; not built)

18. **pacs.008 in, chain out** — ISO from a real core; Quub memo fields already match the stub.
19. **Travel-rule attach** — `trHash` is already on the call; plug TRP/Sumsub instead of `0x00…`.
20. **Maker-checker payments desk** — API + two F211 keys for unfreeze/fee/threshold; freeze still one person.
21. **Correspondent payout** — Bank A originates on Quub; Bank B’s USDC arrives on Base; Quub is the audit spine.
22. **Marketplace settlement** — platform pays vendors as identified memos; 70% lane keeps payouts ahead of bots.
23. **Payroll / disbursement batch** — many memos, same fee token, freeze list for stopped employees.
24. **Escrow-like hold** — freeze beneficiary or sender under policy; not a new escrow VM.
25. **Multi-rail same identity** — later Solana / Ethereum mainnet CCTP; same `endToEndId` + `RailRecord`.
26. **CCIP for non-USDC** — only assets Circle will not carry. Still no Quub-wrapped asset.
27. **Read-only verifier** — third party runs `quub-node` + `op-node` without sequencer key; checks memos and freezes.
28. **Regulator extract** — export F212 packs + F202 hashes + freeze log for a period.
29. **Design-partner sandbox** — TLS API + testnet USDC ≤ $1 + their freeze list.
30. **On-us vs off-us** — on-us settles only on 8091; off-us adds CCTP. Same API body.

### Do not sell as a Quub use case

- Gas token / “buy QUUB to pay”
- General DEX / NFT / “EVM L1 for everything”
- Replacing Circle, replacing Swift, replacing Chainlink
- Atomic DvP with arbitrary L1 assets (no messaging standard locked)
- Privacy pool / mixing
- Consumer retail wallet as the year-1 product
- “All chains” as day-one scope

**One sentence for a partner:** Quub is the policy and identity ledger for a dollar that already lives on someone else’s rail — freeze, memo, evidence, priority block space — not a new coin and not a new bridge.
