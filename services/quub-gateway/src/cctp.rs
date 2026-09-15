//! CCTP V2 pin for Ethereum Sepolia → Base Sepolia (see `specs/rail-cctp.md`).
//!
//! Quub 8091 is **not** a CCTP domain. Burn never targets Quub RPC.

use alloy_primitives::{address, Address, B256};
use serde::{Deserialize, Serialize};

/// Official Circle docs: <https://developers.circle.com/cctp/evm-smart-contracts>
pub const CIRCLE_EVM_CONTRACTS_URL: &str =
    "https://developers.circle.com/cctp/evm-smart-contracts";

pub const ETH_SEPOLIA_DOMAIN: u32 = 0;
pub const BASE_SEPOLIA_DOMAIN: u32 = 6;
pub const ETH_SEPOLIA_CHAIN_ID: u64 = 11155111;
pub const BASE_SEPOLIA_CHAIN_ID: u64 = 84532;
/// Quub identity chain — not a CCTP domain.
pub const QUUB_CHAIN_ID: u64 = 8091;

/// CCTP V2 TokenMessenger (shared across V2 testnets).
pub const TOKEN_MESSENGER_V2: Address =
    address!("0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA");
/// CCTP V2 MessageTransmitter (shared across V2 testnets).
pub const MESSAGE_TRANSMITTER_V2: Address =
    address!("0xE737e5cEBEEBa77EFE34D4aa090756590b1CE275");

pub const USDC_ETH_SEPOLIA: Address =
    address!("0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238");
pub const USDC_BASE_SEPOLIA: Address =
    address!("0x036CbD53842c5426634e7929541eC2318f3dCF7e");

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CctpAddresses {
    pub source_domain: u32,
    pub dest_domain: u32,
    pub source_chain_id: u64,
    pub dest_chain_id: u64,
    pub token_messenger: Address,
    pub message_transmitter: Address,
    pub usdc_source: Address,
    pub usdc_dest: Address,
}

impl Default for CctpAddresses {
    fn default() -> Self {
        Self::sepolia_to_base_sepolia()
    }
}

impl CctpAddresses {
    /// Live pin from Circle's current V2 testnet table (same struct used in mock).
    pub fn sepolia_to_base_sepolia() -> Self {
        Self {
            source_domain: ETH_SEPOLIA_DOMAIN,
            dest_domain: BASE_SEPOLIA_DOMAIN,
            source_chain_id: ETH_SEPOLIA_CHAIN_ID,
            dest_chain_id: BASE_SEPOLIA_CHAIN_ID,
            token_messenger: TOKEN_MESSENGER_V2,
            message_transmitter: MESSAGE_TRANSMITTER_V2,
            usdc_source: USDC_ETH_SEPOLIA,
            usdc_dest: USDC_BASE_SEPOLIA,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurnResult {
    pub burn_tx: B256,
    pub message_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attestation {
    pub attestation: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MintResult {
    pub mint_tx: B256,
}

/// Spy / mode for unit tests and offline rail.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CctpMode {
    #[default]
    Mock,
    /// Live path is gated by env; unit tests stay on Mock.
    Live,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CctpError {
    #[error("burn refused: policy rejected")]
    PolicyRejected,
    #[error("live CCTP not configured in unit tests")]
    LiveNotAvailable,
    #[error("burn id must not equal Quub tx hash")]
    BurnEqualsQuubTx,
}

/// CCTP adapter: burn on Ethereum Sepolia, mint on Base Sepolia.
///
/// Never accepts a Quub (8091) RPC as the burn target.
#[derive(Debug)]
pub struct CctpAdapter {
    pub addrs: CctpAddresses,
    pub mode: CctpMode,
    pub burn_calls: u32,
    /// When true, `burn` returns PolicyRejected (test spy).
    pub refuse_burn: bool,
    mock_nonce: u64,
}

impl Default for CctpAdapter {
    fn default() -> Self {
        Self::mock()
    }
}

impl CctpAdapter {
    pub fn mock() -> Self {
        Self {
            addrs: CctpAddresses::sepolia_to_base_sepolia(),
            mode: CctpMode::Mock,
            burn_calls: 0,
            refuse_burn: false,
            mock_nonce: 1,
        }
    }

    /// `depositForBurn`-shaped burn. Mock returns synthetic ids ≠ any Quub hash passed in.
    /// `quub_tx` is used only to assert burn id ≠ Quub (never packed into the messenger).
    pub fn burn(
        &mut self,
        usdc: Address,
        amount: u128,
        dest_domain: u32,
        mint_recipient: Address,
        quub_tx: B256,
    ) -> Result<BurnResult, CctpError> {
        let _ = (usdc, amount, mint_recipient);
        if self.refuse_burn {
            return Err(CctpError::PolicyRejected);
        }
        if dest_domain != self.addrs.dest_domain {
            // Still allow mock with pinned dest; wrong domain is a programming error in live.
        }
        self.burn_calls += 1;

        match self.mode {
            CctpMode::Mock => {
                let n = self.mock_nonce;
                self.mock_nonce += 1;
                // Distinct from Quub: prefix 0xcc… and encode nonce (never copy quub_tx).
                let mut burn = [0u8; 32];
                burn[0] = 0xcc;
                burn[1] = 0xb1; // burn marker
                burn[24..32].copy_from_slice(&n.to_be_bytes());
                let burn_tx = B256::from(burn);
                if burn_tx == quub_tx {
                    return Err(CctpError::BurnEqualsQuubTx);
                }
                let mut msg = Vec::with_capacity(64);
                msg.extend_from_slice(b"cctp-v2-mock-msg");
                msg.extend_from_slice(&n.to_be_bytes());
                msg.extend_from_slice(quub_tx.as_slice()); // correlation only in mock payload bytes
                Ok(BurnResult {
                    burn_tx,
                    message_bytes: msg,
                })
            }
            CctpMode::Live => Err(CctpError::LiveNotAvailable),
        }
    }

    pub fn attest(&mut self, message_bytes: &[u8]) -> Result<Attestation, CctpError> {
        match self.mode {
            CctpMode::Mock => {
                let mut a = Vec::from(b"mock-attestation:");
                a.extend_from_slice(message_bytes);
                Ok(Attestation { attestation: a })
            }
            CctpMode::Live => Err(CctpError::LiveNotAvailable),
        }
    }

    pub fn mint(
        &mut self,
        message: &[u8],
        attestation: &Attestation,
        quub_tx: B256,
    ) -> Result<MintResult, CctpError> {
        let _ = (message, attestation);
        match self.mode {
            CctpMode::Mock => {
                let mut mint = [0u8; 32];
                mint[0] = 0xcc;
                mint[1] = 0xb2; // mint marker
                mint[2..10].copy_from_slice(&(self.mock_nonce.saturating_sub(1)).to_be_bytes());
                let mint_tx = B256::from(mint);
                if mint_tx == quub_tx {
                    return Err(CctpError::BurnEqualsQuubTx);
                }
                Ok(MintResult { mint_tx })
            }
            CctpMode::Live => Err(CctpError::LiveNotAvailable),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::fixed_bytes;

    #[test]
    fn pin_matches_circle_v2_testnet() {
        let a = CctpAddresses::sepolia_to_base_sepolia();
        assert_eq!(a.source_domain, 0);
        assert_eq!(a.dest_domain, 6);
        assert_eq!(
            a.token_messenger,
            address!("0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA")
        );
        assert_eq!(QUUB_CHAIN_ID, 8091);
        assert_ne!(QUUB_CHAIN_ID, a.source_chain_id);
    }

    #[test]
    fn mock_burn_id_ne_quub_tx() {
        let mut c = CctpAdapter::mock();
        let quub = fixed_bytes!(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        let b = c
            .burn(USDC_ETH_SEPOLIA, 1_000_000, BASE_SEPOLIA_DOMAIN, Address::ZERO, quub)
            .unwrap();
        assert_ne!(b.burn_tx, quub);
    }
}
