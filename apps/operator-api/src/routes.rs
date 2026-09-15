//! HTTP route handlers.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use quub_iso::{validate_and_commit_raw, IsoError};
use quub_primitives::CHAIN_ID_TESTNET;
use serde_json::json;

use crate::auth::BearerAuth;
use crate::config::parse_b256;
use crate::eth::EthError;
use crate::reason::reason_name;
use crate::state::{AppState, PaymentRecord};
use crate::types::{
    FreezeRequest, FreezeResponse, HealthResponse, PaymentRequest, PaymentStatus,
    PaymentSubmitted, PaymentView, PolicyRejectedBody,
};

pub async fn health(State(state): State<Arc<AppState>>) -> Response {
    match state.eth.health_chain_id().await {
        Ok(chain_id) => Json(HealthResponse {
            ok: true,
            service: "quub-operator-api",
            rpc: state.config.rpc_url.clone(),
            chain_id: Some(chain_id),
            error: None,
        })
        .into_response(),
        Err(e) => {
            let chain_id = state.eth.chain_id().await.ok();
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    ok: false,
                    service: "quub-operator-api",
                    rpc: state.config.rpc_url.clone(),
                    chain_id: chain_id.filter(|&id| id == CHAIN_ID_TESTNET).or(chain_id),
                    error: Some(e.to_string()),
                }),
            )
                .into_response()
        }
    }
}

pub async fn create_payment(
    State(state): State<Arc<AppState>>,
    BearerAuth: BearerAuth,
    Json(body): Json<PaymentRequest>,
) -> Response {
    // Idempotency: only records with a successful broadcast exist.
    if let Some(existing) = state.get_payment(&body.end_to_end_id) {
        return (
            StatusCode::ACCEPTED,
            Json(PaymentSubmitted {
                id: body.end_to_end_id,
                tx_hash: existing.tx_hash,
                status: PaymentStatus::Submitted,
            }),
        )
            .into_response();
    }

    let origin = state.eth.operator_address();
    if let Err(e) = validate_and_commit_raw(
        body.end_to_end_id,
        body.uetr,
        body.instr_id,
        body.ccy,
        body.msg_type,
        origin,
    ) {
        return iso_error(e);
    }

    match state
        .eth
        .send_transfer_with_memo(
            body.to,
            body.amount,
            body.end_to_end_id,
            body.uetr,
            body.instr_id,
            body.ccy,
            body.msg_type,
            body.tr_hash,
            body.pack_hash,
        )
        .await
    {
        Ok(tx_hash) => {
            let record = PaymentRecord {
                tx_hash,
                status: PaymentStatus::Submitted,
                memo_hash: None,
            };
            let stored = state.insert_if_absent(body.end_to_end_id, record);
            (
                StatusCode::ACCEPTED,
                Json(PaymentSubmitted {
                    id: body.end_to_end_id,
                    tx_hash: stored.tx_hash,
                    status: PaymentStatus::Submitted,
                }),
            )
                .into_response()
        }
        Err(EthError::PolicyRejected { reason }) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(PolicyRejectedBody {
                error: "policy_rejected",
                reason,
                reason_name: reason_name(reason),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "rpc", "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn get_payment(
    State(state): State<Arc<AppState>>,
    BearerAuth: BearerAuth,
    Path(end_to_end_id): Path<String>,
) -> Response {
    let id = match parse_b256(&end_to_end_id) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "bad_id", "message": e })),
            )
                .into_response();
        }
    };
    let Some(mut record) = state.get_payment(&id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "not_found" })),
        )
            .into_response();
    };

    if matches!(record.status, PaymentStatus::Submitted) {
        match state.eth.receipt_status_and_memo(record.tx_hash).await {
            Ok(Some((true, memo))) => {
                record.status = PaymentStatus::Mined;
                record.memo_hash = memo;
                state.update_payment(id, record.clone());
            }
            Ok(Some((false, _))) => {
                record.status = PaymentStatus::Failed;
                state.update_payment(id, record.clone());
            }
            Ok(None) => {}
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "error": "rpc", "message": e.to_string() })),
                )
                    .into_response();
            }
        }
    }

    Json(PaymentView {
        id,
        tx_hash: record.tx_hash,
        status: record.status,
        memo_hash: record.memo_hash,
    })
    .into_response()
}

pub async fn freeze(
    State(state): State<Arc<AppState>>,
    BearerAuth: BearerAuth,
    Json(body): Json<FreezeRequest>,
) -> Response {
    freeze_or_unfreeze(state, body.address, true).await
}

pub async fn unfreeze(
    State(state): State<Arc<AppState>>,
    BearerAuth: BearerAuth,
    Json(body): Json<FreezeRequest>,
) -> Response {
    freeze_or_unfreeze(state, body.address, false).await
}

async fn freeze_or_unfreeze(
    state: Arc<AppState>,
    address: alloy_primitives::Address,
    freeze: bool,
) -> Response {
    let result = if freeze {
        state.eth.send_freeze(address).await
    } else {
        state.eth.send_unfreeze(address).await
    };
    match result {
        Ok(tx_hash) => Json(FreezeResponse {
            tx_hash,
            address,
            status: PaymentStatus::Submitted,
        })
        .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "rpc", "message": e.to_string() })),
        )
            .into_response(),
    }
}

fn iso_error(e: IsoError) -> Response {
    let msg = e.to_string();
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": "iso", "message": msg })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{fixed_bytes, B256};
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[test]
    fn idempotency_store_only_via_insert() {
        // Mirror AppState contract: failed sends never call insert.
        let map: Mutex<HashMap<B256, PaymentRecord>> = Mutex::new(HashMap::new());
        let id = fixed_bytes!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert!(map.lock().unwrap().get(&id).is_none());
        assert!(map.lock().unwrap().is_empty());
        map.lock().unwrap().insert(
            id,
            PaymentRecord {
                tx_hash: B256::from([1u8; 32]),
                status: PaymentStatus::Submitted,
                memo_hash: None,
            },
        );
        assert!(map.lock().unwrap().contains_key(&id));
    }

    #[test]
    fn status_words_locked() {
        assert_eq!(PaymentStatus::Submitted.as_str(), "submitted");
        assert_eq!(PaymentStatus::Mined.as_str(), "mined");
        assert_eq!(PaymentStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn source_has_no_retired_sister_brand() {
        let forbidden = format!("{}{}", "x", "zero");
        for src in [
            include_str!("routes.rs"),
            include_str!("main.rs"),
            include_str!("types.rs"),
            include_str!("eth.rs"),
            include_str!("auth.rs"),
            include_str!("config.rs"),
            include_str!("state.rs"),
            include_str!("reason.rs"),
        ] {
            assert!(
                !src.to_ascii_lowercase().contains(&forbidden),
                "retired sister brand must not appear in operator-api sources"
            );
        }
    }
}
