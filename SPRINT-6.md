# Quub — Sprint 6
**After:** Sprint 3 lane accepted. Sprint 4/5 may still be open — this sprint must not edit pool, payload, F211 dual-control, or the Reth pin.  
**This sprint:** first public rail. Quub records the identified payment. **CCTP moves USDC.** Base is the first destination chain.

Quub is not a bridge. Quub does not mint USDC. Quub does not replace Circle.

---

## Outcome

```
originator
    →  Quub F210 transferWithMemo + F212 evidence   (policy + identity)
    →  Circle CCTP burn on source
    →  CCTP mint on Base
    →  Quub stores {quubTx, cctpBurn, cctpMint, domain ids}
```

Done when `scripts/rail-base.sh` prints:

- Quub tx hash (8091) status 1 + `memoHash`
- CCTP message bytes / attestation id (live testnet **or** recorded mock)
- Destination: Base Sepolia (live) or `mock://base-sepolia` (offline)
- `GET` (CLI or `services/quub-gateway`) returns all three ids tied to the same `endToEndId`

If there are no Circle/Base RPC credentials, the **mock path is a valid Sprint 6 pass**. The live path is preferred when `CIRCLE_API` + `BASE_SEPOLIA_RPC` are set.

---

## Locked

| Item | Choice |
|---|---|
| First destination | **Base Sepolia** (chain 84532) live; mock if no RPC |
| USDC transport | **Circle CCTP V2** (or current Circle TokenMessenger on that net) |
| Other tokens / CCIP | Not this sprint |
| Solana / Ethereum mainnet | Not this sprint |
| Quub chain | 8091 `--dev` is enough. Mode A 9545 optional. |
| Token | None. Fee still listed F210 on Quub. Destination cash is USDC. |
| Pins | `aef8d3ef`, rustc 1.96, F210–F213, memo hash, 70/30 — do not touch |
| Brand | Quub only. `services/quub-gateway` (rename if still `xzero-*`) |

---

## What Quub owns vs what Circle owns

| Quub | Circle / Base |
|---|---|
| Who may pay (`F201` / freeze) | Burn/mint USDC |
| Payment identity (`F202` memo) | Attestation |
| Evidence hash (`F212`) | Destination USDC credit |
| Idempotency on `endToEndId` | CCTP nonce / message hash |

If policy rejects, **do not** call CCTP. If CCTP fails after Quub mined, record `rail_status=failed` and keep the Quub receipt (do not rewind F210). Retry CCTP with the same `endToEndId`. Do not send a second Quub memo.

---

## Layout

```
services/quub-gateway/     # replace stubs / xzero-orchestrator
  src/lib.rs
  src/cctp.rs              # TokenMessenger encode + Circle attestation client
  src/base.rs              # destination RPC
  src/record.rs            # endToEndId → { quubTx, burnTx, mintTx, memoHash }
apps/operator-api/         # ONLY if Sprint 4 already merged:
  POST /v1/rails/cctp      # thin wrap; do not block Sprint 6 on this
scripts/rail-base.sh
specs/rail-cctp.md
```

No new precompile. No new F2xx address. Destination USDC contract addresses come from Circle’s published Sepolia table — pin them in `specs/rail-cctp.md`, do not hard-code from memory.

---

## Work order

1. **Spec** `specs/rail-cctp.md`: source domain, dest domain (Base Sepolia), TokenMessenger + MessageTransmitter addresses, USDC addresses. Cite Circle docs. Mock mode uses the same struct with `0x` placeholders.
2. **`CctpAdapter`**
   - `burn(usdc, amount, destDomain, mintRecipient) -> message_bytes`
   - `attest(message_bytes) -> attestation` (Circle API **or** mock that returns a fixed attestation)
   - `mint(message, attestation) -> tx_hash`
3. **`RailRecord`** keyed by `endToEndId`: Quub tx + memoHash + burn + mint + status `quub_only | burned | minted | failed`.
4. **`scripts/rail-base.sh`**
   - Requires `--dev` 8545 (start if down).
   - Submit `transferWithMemo` (cast or operator-api if up).
   - If `BASE_SEPOLIA_RPC` + burner key + Circle env present: live CCTP amount **1 USDC or less**.
   - Else: mock burn/mint, write JSON under `artifacts/rail-base/`.
   - Print the four ids.
5. **Tests:** policy reject → adapter not called; idempotent `endToEndId`; mock mint records dest hash.
6. **ADR-020** in ARCHITECTURE.md: first rail = CCTP to Base; Quub does not bridge.

---

## Stop

- No “universal bridge.”
- No CCIP in this PR.
- No Solana adapter (that is a later rail sprint).
- No Quub-wrapped USDC.
- No new chain id on Quub.
- No Reth bump, Simplex, ticker.
- Do not edit `quub-pool` / `quub-payload`.
- Do not send mainnet USDC.

---

## Acceptance

- `cargo test -p quub-gateway` (or the crate name) green; workspace + forge still green.
- `scripts/rail-base.sh` prints Quub tx + memoHash + burn id + mint id (mock ids allowed).
- Policy reject fixture never calls burn.
- `specs/rail-cctp.md` has real Circle addresses for Base Sepolia (even if run is mock).
- `rg xzero services/quub-gateway apps/operator-api` empty on files you touch.

Print: mode `live|mock`, Quub tx, memoHash, burn id, mint id, cargo + forge counts.

---

## Cursor first message

```
Read AGENTS.md, ARCHITECTURE.md, SPRINT-6.md.

Sprint 6: first public rail. CCTP carries USDC to Base Sepolia.
Quub records policy + memo + evidence. Quub is not a bridge.

Do not edit quub-pool, quub-payload, launch.rs, or the Reth pin.
No Simplex. No token. No CCIP. No Solana. No mainnet.

1. specs/rail-cctp.md with Circle TokenMessenger / USDC / domain ids
   for Base Sepolia (cite docs).
2. services/quub-gateway CctpAdapter + RailRecord keyed by endToEndId.
3. scripts/rail-base.sh: Quub memo on 8545, then CCTP live if env is set,
   else mock and write artifacts/rail-base/.
4. Policy reject must not call burn. Idempotent endToEndId.
5. ADR-020.

If CIRCLE_API / BASE_SEPOLIA_RPC missing, mock is a pass.

Stop and print: live|mock, Quub tx, memoHash, burn id, mint id, test counts.
```
