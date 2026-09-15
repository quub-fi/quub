# Quub — Deployment architecture
**Status:** locked through Sprint 5; Sprint 6 rail is the last year-1 *chain* piece.  
**Domains:** `quub.network` = protocol / RPC. `quub.fi` = company / product.  
**Chain id:** 8091 on every Quub execution path.  
**Pin:** `op-rs/reth` rev `aef8d3ef92117f91455e16969f0adf5bf7c6e9e1`, `op-reth/v2.4.4`, `op-node` v1.19.7, rustc 1.96.

This document is how Quub is **run**. Protocol decisions live in `ARCHITECTURE.md` / ADRs.

---

## 1. What is deployed vs what is not

| Exists today (laptop / local) | Not deployed |
|---|---|
| `quub-node --dev` on `:8545` | Public RPC / TLS |
| Mode A: geth L1 `:8546` + `quub-node --engine` `:9545/:9551` + `op-node` v1.19.7 | Batcher / proposer to a public L1 |
| `apps/operator-api` `:8080` | `api.quub.network` |
| F210–F213 etched at genesis | Superchain registry |
| Payment lane 70/30 in the payload builder | Mode B / Simplex |
| Paymaster `takeFee` + dual-control on `--dev` | Native token |
| | KMS / HSM keys |
| | Postgres idempotency |
| | Live CCTP (Sprint 6 mock is the offline path) |

Do not publish `--dev` or anvil keys. Do not point `quub.network` at `127.0.0.1`.

---

## 2. Environments

```
┌─────────────────────────────────────────────────────────────┐
│  DEV (engineer laptop)                                      │
│  --dev :8545 + operator-api :8080                           │
│  OwnerA=anvil0  OwnerB=anvil1  feeRecipient=0x…FEE0         │
│  Datadir ephemeral unless scripts set a named path          │
└─────────────────────────────────────────────────────────────┘
                              │
                              │  same binary, different launch
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  MODE A LOCAL (derived L2)                                  │
│  geth L1 :8546  (full genesis, SystemConfig 0xe991…F990)    │
│  quub-node --engine :9545 / authrpc :9551 + JWT             │
│  op-node v1.19.7                                            │
│  L2 genesis = OP alloc ∪ Quub F210–F213                     │
│  OwnerB may still equal OwnerA (STATUS.md exception)        │
└─────────────────────────────────────────────────────────────┘
                              │
                              │  not built yet
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  TESTNET (next)                                             │
│  Public L1 = Sepolia                                        │
│  op-batcher + op-proposer                                   │
│  quub-node --engine + op-node                               │
│  operator-api behind TLS                                    │
│  CCTP Sepolia → Base Sepolia (≤ 1 USDC)                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  PARTNER / MAINNET (later)                                  │
│  Public L1 = Ethereum                                       │
│  Destination cash = USDC on Base (CCTP)                     │
│  Quub still 8091 as the policy/identity ledger              │
│  Keys in KMS. Freeze instant. Params + unfreeze = 2 keys    │
└─────────────────────────────────────────────────────────────┘
```

Three launch paths, one execution crate (`QuubEvmFactory`). `--dev` is not Mode A with `op-node` killed.

---

## 3. Process view (Mode A local — the real stack)

```
                    JWT  artifacts/mode-a/jwt.txt
                              │
 ops / curl / operator-api    │
        │                     │
        │ HTTP :8080          │
        ▼                     │
 operator-api                 │
        │ eth_*               │
        ▼                     │
 quub-node --engine ◄─────────┤ Engine API :9551
 HTTP :9545                   │
 chain 8091                   │
 QuubEvmFactory               │
 F201–F203 precompiles        │
 F210–F213 genesis            │
 QuubLaneTxs 70/30            │
        ▲                     │
        │ derive              │
        │                     ▼
     op-node v1.19.7      rollup.json
        │                     ▲
        │ L1 RPC :8546        │
        ▼                     │
     geth 1.16.x              │
     --datadir artifacts/mode-a/l1
     genesis = op-deployer 0.8.x
     SystemConfig 0xe991d1A4…FC990
```

`--dev` is the same `quub-node` binary without `op-node` / geth: local miner, HTTP `:8545`, same alloc.

**Kill rule:** stop `op-node` → L2 block number must freeze. If it keeps moving, you are on the `--dev` miner and Mode A is a lie.

---

## 4. Port map

| Port | Process | Role |
|---|---|---|
| 8545 | `quub-node --dev` | Engineer RPC. Default `QUUB_RPC`. |
| 8546 | geth L1 | Mode A settlement L1. Not anvil. |
| 9545 | `quub-node --engine` | L2 HTTP. |
| 9551 | `quub-node --engine` | Engine API (JWT). |
| 8080 | `operator-api` | HTTP product surface. Bind `127.0.0.1` until TLS exists. |

Do not run `--dev` and `--engine` as two miners on the same datadir. Two processes, two datadirs.

---

## 5. Genesis and addresses

Frozen on every environment that runs Quub execution:

| Addr | Role |
|---|---|
| `0x…F201` | Policy precompile (stateful, F211 sload, caller = F210) |
| `0x…F202` | ISO memo precompile (stateful, origin in hash) |
| `0x…F203` | Paymaster precompile (thin; F213 owns the move) |
| `0x…F210` | PaymentToken (genesis alloc) |
| `0x…F211` | PolicyAdmin |
| `0x…F212` | EvidenceAnchor |
| `0x…F213` | PaymasterEntry |

`--dev` owners (Sprint 5):

- OwnerA = anvil0 `0xf39F…2266`
- OwnerB = anvil1 `0x7099…79C8`
- feeRecipient = `0x…FEE0` (dev sink, not a ticker)

Mode A L2 alloc may still have OwnerA = OwnerB. That is a known exception (`STATUS.md`). Do not ship partner Mode A until OwnerB is etched in `l2-genesis-quub.json`.

Rebuild alloc with `forge inspect` + hex in `crates/quub-node/alloc/`. Never `CREATE` onto `0x…F210`.

---

## 6. Data flow (a payment)

```
Client
  POST /v1/payments   Bearer QUUB_OPERATOR_TOKEN
        │
        ▼
operator-api
  quub-iso validate (endToEndId, UETR, …)
  idempotency map[endToEndId]     ← memory today; Postgres next
  sign with QUUB_OPERATOR_KEY
  eth_sendRawTransaction
        │
        ▼
quub-node
  pool classify: F210 + transferWithMemo → payment lane (70%)
  F201 check F211 (freeze / travel-rule / pause)
  F213 takeFee (listed token, caller F210 only)
  F210 principal _move + MemoAnchored
  F202 memoHash = keccak256(abi.encode(…, tx.origin))
  F212 EvidenceAnchored(packHash, memoHash)
        │
        ▼
[Sprint 6] quub-gateway
  if policy ok → CCTP burn (Sepolia USDC) → attest → mint Base Sepolia
  RailRecord[endToEndId] = { quubTx, memoHash, burn, mint, status }
  Quub 8091 is NOT a CCTP domain. Do not burn QPT.
```

Freeze: `POST /v1/freeze` → F211 `freeze` (either owner, immediate).  
Unfreeze: propose OwnerA + confirm OwnerB. API must not send a single-key `unfreeze` after Sprint 5.

---

## 7. Scripts (how we actually start things)

| Script | Stack |
|---|---|
| `scripts/devnet.sh` | `--dev` :8545, one memo |
| `scripts/mode-a.sh` | geth + `--engine` + `op-node`, L2 memo + freeze + kill-test |
| `scripts/payment-lane-flood.sh` | mixed flood, print 70/30 share |
| `scripts/sprint5-dev.sh` | fee delta + two-key unfreeze |
| `scripts/gateway-dev.sh` | API :8080 pay / freeze / 422 |
| `scripts/rail-base.sh` | Quub memo then CCTP live \| mock |

Operator install (dev):

```
cargo build -p quub-node -p quub-operator-api
export PATH="$(pwd)/target/debug:$PATH"
export QUUB_RPC=http://127.0.0.1:8545
export QUUB_OPERATOR_TOKEN=…
export QUUB_OPERATOR_KEY=…    # anvil0 locally only
```

`op-node` v1.19.7 is source-built today (no GitHub binary on that tag). geth 1.16.x and `op-deployer` 0.8.x live under `artifacts/mode-a/bin/` on the laptop that ran Sprint 2. Document those paths in README; do not assume Homebrew `op-node`.

---

## 8. Secrets

| Secret | Dev | Next |
|---|---|---|
| anvil0 / anvil1 private keys | Well-known; `--dev` only | Delete from runbooks |
| `QUUB_OPERATOR_KEY` | anvil0 | KMS signer |
| `QUUB_OPERATOR_TOKEN` | env string | rotated bearer or mTLS |
| Engine JWT | `artifacts/mode-a/jwt.txt` | generated per env, 0600 |
| OwnerA / OwnerB | anvil keys | two people / two HSMs |
| CCTP burner | unset → mock | funded Sepolia key, ≤ 1 USDC |
| Circle API | unset → mock | server-side only |

Never commit JWT, operator token, or funded keys. `--dev` keys in scripts are allowed only with a comment that they are anvil fixtures.

---

## 9. Target production shape (not built)

When a partner exists:

1. **L1:** Ethereum (Sepolia first). `op-deployer` genesis with Quub L2 alloc merged.  
2. **L2 sequencers:** two `quub-node --engine` behind a private Engine API. One `op-node` sequencer, one verifier.  
3. **Batcher / proposer:** standard OP images, same versions as `op-node` v1.19.7 line (bump as a set, never Reth alone).  
4. **RPC:** `rpc.quub.network` → `eth_*` only. No `quub_` namespace year-1.  
5. **API:** `api.quub.network` → `operator-api` + TLS. Postgres for `endToEndId`.  
6. **Rail:** `quub-gateway` workers, not in the node process. CCTP first; CCIP later.  
7. **Observability:** chain id, head lag vs `op-node`, F211 freeze events, rail `failed` count, feeRecipient balance.  
8. **Domains:** A records only after TLS. `quub.fi` stays marketing.

Quub does not custody USDC. Circle does. Quub custody is the operator key that can freeze and submit memos.

---

## 10. Failure modes

| Failure | What should happen |
|---|---|
| `op-node` down | L2 stops. API `/health` not `ok` if `QUUB_RPC` is 9545. |
| `--dev` datadir wiped | Receipts vanish. Expected. Named datadir if you need history. |
| Policy reject | No principal, no fee, no CCTP burn. |
| CCTP down after Quub mined | `RailRecord.status=failed`. Retry rail. No second memo. |
| Operator key leaked | Freeze that address. Rotate token. Do not “upgrade” F210. |
| Wrong chain id | `/health` fail-closed. Never send to 1 / 8453 / 31337 by accident. |

---

## 11. Decision log (deployment)

| ADR | Deploy meaning |
|---|---|
| 016 | Execution pin is `op-rs/reth` + `op-reth/v2.4.4`. One Reth remote. |
| 017 | Payload builder is Quub on both `--dev` and `--engine`. |
| 019 | `--dev` two owners + live `takeFee`. Mode A owners still TBD. |
| 020 | First public rail CCTP → Base. Quub is not a CCTP domain. |

If a deploy needs a second Reth git, a new chain id, a ticker, or Simplex in the same binary — it is out of architecture, not a hotfix.
