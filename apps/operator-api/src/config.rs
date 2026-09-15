//! Runtime configuration from env.

use alloy_primitives::{keccak256, Address, B256};
use k256::ecdsa::SigningKey;
use quub_primitives::CHAIN_ID_TESTNET;

#[derive(Clone)]
pub struct Config {
    pub rpc_url: String,
    pub operator_token: String,
    pub signing_key: SigningKey,
    pub operator_address: Address,
    pub chain_id: u64,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("rpc_url", &self.rpc_url)
            .field("operator_address", &self.operator_address)
            .field("chain_id", &self.chain_id)
            .finish_non_exhaustive()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let rpc_url =
            std::env::var("QUUB_RPC").unwrap_or_else(|_| "http://127.0.0.1:8545".to_string());
        let operator_token = std::env::var("QUUB_OPERATOR_TOKEN")
            .map_err(|_| "QUUB_OPERATOR_TOKEN is required".to_string())?;
        if operator_token.is_empty() {
            return Err("QUUB_OPERATOR_TOKEN must be non-empty".into());
        }
        let key = std::env::var("QUUB_OPERATOR_KEY")
            .map_err(|_| "QUUB_OPERATOR_KEY is required".to_string())?;
        let signing_key = parse_signing_key(&key)?;
        let operator_address = address_from_key(&signing_key);
        Ok(Self {
            rpc_url,
            operator_token,
            signing_key,
            operator_address,
            chain_id: CHAIN_ID_TESTNET,
        })
    }
}

pub fn parse_signing_key(hex_key: &str) -> Result<SigningKey, String> {
    let hex_key = hex_key.trim().trim_start_matches("0x");
    let bytes = hex::decode(hex_key).map_err(|e| format!("invalid QUUB_OPERATOR_KEY hex: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!(
            "QUUB_OPERATOR_KEY must be 32 bytes, got {}",
            bytes.len()
        ));
    }
    SigningKey::from_slice(&bytes).map_err(|e| format!("invalid QUUB_OPERATOR_KEY: {e}"))
}

pub fn address_from_key(key: &SigningKey) -> Address {
    let verifying = key.verifying_key();
    let uncompressed = verifying.to_encoded_point(false);
    let hash = keccak256(&uncompressed.as_bytes()[1..]);
    Address::from_slice(&hash[12..])
}

/// Parse a 0x-prefixed 32-byte hex id.
pub fn parse_b256(s: &str) -> Result<B256, String> {
    s.parse::<B256>()
        .map_err(|e| format!("invalid bytes32: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quub_primitives::DEVNET_ANVIL_ADDRESS;

    #[test]
    fn anvil0_key_derives_known_address() {
        let key = parse_signing_key(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        )
        .unwrap();
        assert_eq!(address_from_key(&key), DEVNET_ANVIL_ADDRESS);
    }
}
