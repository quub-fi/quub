# Quub — Sprint 7
**After:** Sprints 0–6 accepted (including CCTP rail).  
**This sprint:** make a partner demo tell the truth. Four gaps. No new rail, no FX, no agents, no Simplex, no token.

---

## Outcome

1. `POST /v1/unfreeze` is propose + **other owner** confirm (anvil1). Single-key unfreeze does not return 200.  
2. Mode A L2 genesis OwnerB = anvil1 (`0x7099…79C8`). Dual-control works on **9545**, not only `--dev`.  
3. `--dev` and Mode A use **named datadirs**. Restart keeps receipt hashes unless `QUUB_RESET=1`.  
4. `POST /v1/payments` accepts `"rail": "quub" | "cctp"` (default `quub`). `cctp` runs the Sprint 6 gateway path and returns burn/mint ids (mock allowed).

---

## Locked

| Item | Value |
|---|---|
| Pins | `op-rs/reth@aef8d3ef`, `op-reth/v2.4.4`, rustc 1.96, `op-node` v1.19.7 |
| Chain id | 8091 |
| F201–F213 | unchanged |
| Memo hash | `keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, tx.origin))` |
| Lane | `PAYMENT_LANE_BPS = 7000` — do not edit pool/payload fill math |
| F211 | freeze instant; unfreeze = other owner (already on `--dev`) |

---

## 7.1 API two-key unfreeze (C033)

`apps/operator-api`:

- Env: `QUUB_OWNER_A_KEY` (anvil0), `QUUB_OWNER_B_KEY` (anvil1). Operator key may stay anvil0 for **payments**.  
- `POST /v1/unfreeze { address }` does:
  1. `proposeUnfreeze` from A  
  2. `confirmUnfreeze` from B  
  3. return both tx hashes  
- If A==B keys configured, **refuse** (except a documented `--dev` override `QUUB_ALLOW_SAME_OWNER=1` that prints a warning). Default off.  
- Single `unfreeze()` send from the operator key must not be the implementation.

`scripts/gateway-dev.sh`: freeze → 422 reason 1 → `POST /v1/unfreeze` → third payment mines.

---

## 7.2 Mode A OwnerB (C034)

- Merge OwnerB = anvil1 into `l2-genesis-quub.json` / alloc used by `mode-a.sh`.  
- Do **not** hand-edit SystemConfig / portal. Only F211 storage slot for OwnerB.  
- `scripts/mode-a-owners.sh` (or a step in `mode-a.sh`): on **9545**, freeze from anvil0, unfreeze from anvil0 alone reverts, confirm from anvil1 works.  
- Update `STATUS.md`: remove “Mode A still same-key” if the slot is etched.

If op-deployer regen would smash F210–F213, stop and write STATUS. Do not recreate anvil-L1.

---

## 7.3 Named datadirs (C035)

| Env | Default dir |
|---|---|
| `--dev` | `artifacts/datadir-dev` |
| Mode A L2 | `artifacts/datadir-mode-a` |
| geth L1 | keep `artifacts/mode-a/l1` |

- Scripts pass `--datadir` (or equivalent).  
- Wipe only if `QUUB_RESET=1`.  
- Acceptance: mine a memo, restart `--dev`, `cast receipt $HASH` still works.

---

## 7.4 `rail` field (C048)

```json
{ "...existing payment body...", "rail": "quub" }
{ "...existing payment body...", "rail": "cctp" }
```

- Default `quub` = today’s path.  
- `cctp` = after mined memo, call `quub-gateway` (Sprint 6). Return `{ txHash, memoHash, burnTx, mintTx, railStatus }`.  
- Mock if Circle env missing.  
- Policy reject ⇒ no burn (already Sprint 6).  
- Same `endToEndId` + `rail=cctp` twice ⇒ same `RailRecord`, no second memo, no second burn.

Do not add `solana` / `ccip` values this sprint.

---

## Stop

- No Simplex, ticker, new chain id, Reth bump.  
- No AgentRegistry / `msgType=2` runtime (spec-only if you must touch specs).  
- No FX / CAD issuer.  
- No `0.0.0.0`.  
- Do not retune `PAYMENT_LANE_BPS`.  
- Do not put 70/30 in PolicyAdmin.

---

## Acceptance print

- gateway-dev: freeze → 422 → unfreeze (two hashes) → pay mines  
- mode-a 9545: OwnerB ≠ OwnerA **yes**; anvil0-alone unfreeze rejected  
- datadir: receipt survives restart  
- POST rail=quub vs rail=cctp: cctp returns four ids (mock ok)  
- cargo + forge green  

Write `artifacts/sprint7-validate.txt`.

---

## Cursor first message

```
Read AGENTS.md, ARCHITECTURE.md, SPRINT-7.md, ADR-019, ADR-020.

Sprint 6 is accepted. Do Sprint 7 only.

1. POST /v1/unfreeze = proposeUnfreeze from OWNER_A then confirmUnfreeze
   from OWNER_B. Two different keys. No single-key unfreeze.
2. Mode A L2 genesis OwnerB = anvil1. Prove freeze/unfreeze on 9545.
   Do not hand-edit SystemConfig. Do not use anvil as L1.
3. Named datadirs artifacts/datadir-dev and artifacts/datadir-mode-a.
   QUUB_RESET=1 to wipe. Receipt must survive restart.
4. POST /v1/payments rail=quub|cctp (default quub). cctp uses Sprint 6
   gateway; mock allowed. Idempotent endToEndId.

Pins stay aef8d3ef / rustc 1.96 / 8091 / F210–F213 / memo hash.
No Simplex. No token. No FX. No agents. No lane edits.

Stop and print: unfreeze tx hashes, Mode A OwnerB≠OwnerA, receipt after
restart, rail=cctp four ids, cargo+forge counts.
Write artifacts/sprint7-validate.txt.
```
