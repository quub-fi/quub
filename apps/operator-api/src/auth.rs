//! Bearer auth for `/v1/*`.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use crate::state::AppState;
use std::sync::Arc;

pub struct BearerAuth;

impl FromRequestParts<Arc<AppState>> for BearerAuth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());
        let Some(header) = header else {
            return Err(unauthorized("missing Authorization header"));
        };
        let Some(token) = header.strip_prefix("Bearer ") else {
            return Err(unauthorized("expected Bearer token"));
        };
        if token != state.config.operator_token {
            return Err(unauthorized("invalid token"));
        }
        Ok(BearerAuth)
    }
}

fn unauthorized(msg: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "unauthorized", "message": msg })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn bearer_prefix_required() {
        let h = HeaderValue::from_static("Bearer secret");
        let s = h.to_str().unwrap();
        assert_eq!(s.strip_prefix("Bearer ").unwrap(), "secret");
        assert!("Token secret".strip_prefix("Bearer ").is_none());
    }
}
