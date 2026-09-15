//! Shared primitives for Quub ledger and xZERO off-chain libs.
//!
//! Addresses are frozen (AGENTS.md §2). Do not change them.

use alloy_primitives::{address, Address, B256, FixedBytes, U256};

/// Placeholder chain ids (confirm unused on chainid.network before treating as real).
pub const CHAIN_ID_MAINNET: u64 = 8090;
pub const CHAIN_ID_TESTNET: u64 = 8091;

// --- Precompiles ---
pub const QUUB_POLICY: Address = address!("0x000000000000000000000000000000000000F201");
pub const QUUB_ISO_MEMO: Address = address!("0x000000000000000000000000000000000000F202");
pub const QUUB_PAYMASTER: Address = address!("0x000000000000000000000000000000000000F203");

// --- Reserved ---
pub const QUUB_FEE_MANAGER: Address = address!("0x000000000000000000000000000000000000F204");
pub const QUUB_LANE_METER: Address = address!("0x000000000000000000000000000000000000F205");

// --- Solidity system contracts ---
pub const PAYMENT_TOKEN: Address = address!("0x000000000000000000000000000000000000F210");
pub const POLICY_ADMIN: Address = address!("0x000000000000000000000000000000000000F211");
pub const EVIDENCE_ANCHOR: Address = address!("0x000000000000000000000000000000000000F212");
pub const PAYMASTER_ENTRY: Address = address!("0x000000000000000000000000000000000000F213");

/// Dummy fee-token address for F203 `quote` unit tests. Not an alloc. Not USDC.
pub const FEE_TOKEN_DEVNET: Address = address!("0x000000000000000000000000000000000000FEE3");

/// Host allowed to call F203 `takeFee` in Sprint 1 (stateful debit waits for 1.5).
pub const PAYMASTER_TEST_HOST: Address = PAYMASTER_ENTRY;

// --- PaymentToken.sol ABI calldata lengths (selector + padded words) ---
// Measured from contracts/src/PaymentToken.sol. Each ABI word is 32 bytes.

/// `transfer(address,uint256)` — 2 words.
pub const TRANSFER_CALLDATA_LEN: usize = 4 + 32 * 2;
/// `transferFrom(address,address,uint256)` — 3 words.
pub const TRANSFER_FROM_CALLDATA_LEN: usize = 4 + 32 * 3;
/// `transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32)` — 8 words.
pub const TRANSFER_WITH_MEMO_CALLDATA_LEN: usize = 4 + 32 * 8;

/// ERC-20 `transfer(address,uint256)` selector.
pub const TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];
/// ERC-20 `transferFrom(address,address,uint256)` selector.
pub const TRANSFER_FROM_SELECTOR: [u8; 4] = [0x23, 0xb8, 0x72, 0xdd];
/// `transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32)`.
/// `cast sig` of that signature: `0xa3124283`.
pub const TRANSFER_WITH_MEMO_SELECTOR: [u8; 4] = [0xa3, 0x12, 0x42, 0x83];

/// F201 policy `check` reason codes.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyReason {
    Allow = 0,
    FrozenFrom = 1,
    FrozenTo = 2,
    NotAllowlisted = 3,
    AmountOverLimit = 4,
    MissingTravelRule = 5,
    DualControlRequired = 6,
    Paused = 7,
    Malformed = 8,
}

impl PolicyReason {
    pub const fn as_u16(self) -> u16 {
        self as u16
    }
}

/// F202 ISO memo message types.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgType {
    Pacs008 = 0,
    Pain001 = 1,
    Pacs009 = 2,
    Camt054 = 3,
}

impl MsgType {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Pacs008),
            1 => Some(Self::Pain001),
            2 => Some(Self::Pacs009),
            3 => Some(Self::Camt054),
            _ => None,
        }
    }

    /// pacs.008 and pacs.009 require a non-zero UETR.
    pub const fn requires_uetr(self) -> bool {
        matches!(self, Self::Pacs008 | Self::Pacs009)
    }
}

/// Year-1 allowlisted ISO currency codes (ASCII bytes3).
pub const CCY_USD: FixedBytes<3> = FixedBytes([b'U', b'S', b'D']);
pub const CCY_CAD: FixedBytes<3> = FixedBytes([b'C', b'A', b'D']);
pub const CCY_AED: FixedBytes<3> = FixedBytes([b'A', b'E', b'D']);
pub const CCY_SAR: FixedBytes<3> = FixedBytes([b'S', b'A', b'R']);

pub fn is_allowed_ccy(ccy: FixedBytes<3>) -> bool {
    ccy == CCY_USD || ccy == CCY_CAD || ccy == CCY_AED || ccy == CCY_SAR
}

/// Off-chain / on-chain memo identity set. No PII, no timestamp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Memo {
    pub end_to_end_id: B256,
    pub uetr: FixedBytes<16>,
    pub instr_id: B256,
    pub ccy: FixedBytes<3>,
    pub msg_type: MsgType,
}

impl Memo {
    pub fn new(
        end_to_end_id: B256,
        uetr: FixedBytes<16>,
        instr_id: B256,
        ccy: FixedBytes<3>,
        msg_type: MsgType,
    ) -> Self {
        Self {
            end_to_end_id,
            uetr,
            instr_id,
            ccy,
            msg_type,
        }
    }
}

/// Inputs to F201 `check`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyCheck {
    pub token: Address,
    pub from: Address,
    pub to: Address,
    pub amount: U256,
    pub tr_hash: B256,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_addresses_match_agents() {
        assert_eq!(
            format!("{QUUB_POLICY:?}").to_lowercase(),
            "0x000000000000000000000000000000000000f201"
        );
        assert_eq!(
            format!("{QUUB_ISO_MEMO:?}").to_lowercase(),
            "0x000000000000000000000000000000000000f202"
        );
        assert_eq!(
            format!("{QUUB_PAYMASTER:?}").to_lowercase(),
            "0x000000000000000000000000000000000000f203"
        );
        assert_eq!(
            format!("{PAYMENT_TOKEN:?}").to_lowercase(),
            "0x000000000000000000000000000000000000f210"
        );
        assert_eq!(
            format!("{POLICY_ADMIN:?}").to_lowercase(),
            "0x000000000000000000000000000000000000f211"
        );
    }

    #[test]
    fn chain_id_placeholders() {
        assert_eq!(CHAIN_ID_MAINNET, 8090);
        assert_eq!(CHAIN_ID_TESTNET, 8091);
    }

    #[test]
    fn payment_token_abi_lengths_match_solidity() {
        assert_eq!(TRANSFER_CALLDATA_LEN, 68);
        assert_eq!(TRANSFER_FROM_CALLDATA_LEN, 100);
        assert_eq!(TRANSFER_WITH_MEMO_CALLDATA_LEN, 260);
        let h = alloy_primitives::keccak256(
            b"transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32)",
        );
        assert_eq!(&TRANSFER_WITH_MEMO_SELECTOR, &h[..4]);
    }
}
