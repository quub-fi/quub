//! Base Sepolia destination config (CCTP mint target).

use crate::cctp::{BASE_SEPOLIA_CHAIN_ID, BASE_SEPOLIA_DOMAIN, USDC_BASE_SEPOLIA};
use alloy_primitives::Address;
use serde::{Deserialize, Serialize};

pub const MOCK_BASE_URI: &str = "mock://base-sepolia";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseDest {
    pub chain_id: u64,
    pub cctp_domain: u32,
    pub usdc: Address,
    /// Live RPC URL or [`MOCK_BASE_URI`].
    pub rpc: String,
}

impl BaseDest {
    pub fn mock() -> Self {
        Self {
            chain_id: BASE_SEPOLIA_CHAIN_ID,
            cctp_domain: BASE_SEPOLIA_DOMAIN,
            usdc: USDC_BASE_SEPOLIA,
            rpc: MOCK_BASE_URI.to_string(),
        }
    }

    pub fn live(rpc: impl Into<String>) -> Self {
        Self {
            chain_id: BASE_SEPOLIA_CHAIN_ID,
            cctp_domain: BASE_SEPOLIA_DOMAIN,
            usdc: USDC_BASE_SEPOLIA,
            rpc: rpc.into(),
        }
    }

    pub fn is_mock(&self) -> bool {
        self.rpc == MOCK_BASE_URI || self.rpc.starts_with("mock://")
    }
}
