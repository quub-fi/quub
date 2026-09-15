//! Runtime configuration from env.

use alloy::signers::local::PrivateKeySigner;
use alloy_primitives::B256;
use quub_primitives::CHAIN_ID_TESTNET;

#[derive(Clone, Debug)]
pub struct Config {
    pub rpc_url: String,
    pub operator_token: String,
    pub signer: PrivateKeySigner,
    pub chain_id: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let rpc_url = std::env::var("QUUB_RPC")
            .unwrap_or_else(|_| "http://127.0.0.1:8545".to_string());
        let operator_token = std::env::var("QUUB_OPERATOR_TOKEN")
            .map_err(|_| "QUUB_OPERATOR_TOKEN is required".to_string())?;
        if operator_token.is_empty() {
            return Err("QUUB_OPERATOR_TOKEN must be non-empty".into());
        }
        let key = std::env::var("QUUB_OPERATOR_KEY")
            .map_err(|_| "QUUB_OPERATOR_KEY is required".to_string())?;
        let signer: PrivateKeySigner = key
            .parse()
            .map_err(|e| format!("invalid QUUB_OPERATOR_KEY: {e}"))?;
        Ok(Self {
            rpc_url,
            operator_token,
            signer,
            chain_id: CHAIN_ID_TESTNET,
        })
    }
}

/// Parse a 0x-prefixed 32-byte hex id.
pub fn parse_b256(s: &str) -> Result<B256, String> {
    s.parse::<B256>()
        .map_err(|e| format!("invalid bytes32: {e}"))
}
