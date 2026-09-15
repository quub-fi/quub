//! Quub operator API — HTTP in front of `eth_*` (F210 payments, F211 freeze).

mod auth;
mod config;
mod eth;
mod reason;
mod routes;
mod state;
mod types;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::eth::EthClient;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = Config::from_env()?;
    let eth = EthClient::connect(&config).await?;
    let state = Arc::new(AppState::new(config.clone(), eth));

    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/v1/payments", post(routes::create_payment))
        .route("/v1/payments/{end_to_end_id}", get(routes::get_payment))
        .route("/v1/freeze", post(routes::freeze))
        .route("/v1/unfreeze", post(routes::unfreeze))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!(%addr, rpc = %config.rpc_url, "quub-operator-api listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
