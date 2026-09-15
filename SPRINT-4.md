Sprint 4 is the gateway. It runs next to Sprint 3. It does not wait on 70/30. Do not edit quub-pool or quub-payload in this track.
textops → HTTP :8080 → eth\_\* → quub-node :8545 or :9545
F210 payment / F211 freeze
Locked

Brand is Quub only. No xZERO in routes or JSON.
127.0.0.1:8080. Env QUUB_RPC (default http://127.0.0.1:8545), QUUB_OPERATOR_TOKEN, QUUB_OPERATOR_KEY (anvil0 on --dev).
Idempotent on endToEndId.
Reuse services/quub-iso and policy reason codes.
In-memory idempotency. No Postgres. No CCTP (Sprint 6). No Simplex. No token. No Reth bump.

Routes
textGET /health
POST /v1/payments → 202 { id, txHash, status }
GET /v1/payments/{endToEndId}
POST /v1/freeze { address }
POST /v1/unfreeze { address }
Body for pay: to, amount, endToEndId, uetr, instrId, ccy, msgType, trHash, packHash.
Acceptance
scripts/gateway-dev.sh prints: payment tx hash, mined, memoHash, freeze tx, second payment rejected (reason 1). rg xzero apps/operator-api is empty. Workspace tests still green.

Paste into Cursor (Sprint 4 tree only)
text

Read AGENTS.md, ARCHITECTURE.md.

Sprint 4 in parallel with Sprint 3.
Do not touch quub-pool, quub-payload, launch.rs, or the Reth pin.
No Simplex. No token. No CCTP. No xZERO names. Chain id stays 8091.

Build apps/operator-api (Rust, workspace member) on 127.0.0.1:8080.

GET /health
POST /v1/payments (sign transferWithMemo with QUUB_OPERATOR_KEY)
GET /v1/payments/{endToEndId}
POST /v1/freeze
POST /v1/unfreeze

Bearer QUUB_OPERATOR_TOKEN. Idempotent on endToEndId.
Reuse services/quub-iso and reason codes. In-memory idempotency only.
QUUB_RPC default http://127.0.0.1:8545.

scripts/gateway-dev.sh: start --dev if needed, POST payment, GET mined + memoHash,
freeze, second payment rejected.

Stop and print: health JSON, payment tx hash, memoHash, freeze result,
cargo test counts. rg xzero in apps/operator-api must be empty.
Write artifacts/sprint4-validate.txt.
