//! PolicyAdmin (F211) storage layout — from `forge inspect PolicyAdmin storageLayout`.
//!
//! Slots (inspect JSON wins):
//! - 0: ownerA
//! - 1: ownerB (bytes 0..19) + paused (byte 20)
//! - 2: threshold
//! - 3: feeToken
//! - 4: frozen mapping base
//! - 5: proposals mapping base

#![allow(dead_code)] // layout constants kept next to forge inspect JSON

use alloy_primitives::{keccak256, Address, B256, U256};

#[allow(dead_code)] // layout constants from forge inspect JSON
pub const OWNER_A_SLOT: u64 = 0;
pub const OWNER_B_PAUSED_SLOT: u64 = 1;
pub const THRESHOLD_SLOT: u64 = 2;
#[allow(dead_code)]
pub const FEE_TOKEN_SLOT: u64 = 3;
pub const FROZEN_MAPPING_BASE: u64 = 4;

/// `keccak256(abi.encode(account, FROZEN_MAPPING_BASE))`
pub fn frozen_slot(account: Address) -> B256 {
    let mut buf = [0u8; 64];
    buf[12..32].copy_from_slice(account.as_slice());
    buf[56..64].copy_from_slice(&FROZEN_MAPPING_BASE.to_be_bytes());
    keccak256(buf)
}

#[allow(dead_code)]
pub fn slot_u256(slot: u64) -> B256 {
    B256::from(U256::from(slot))
}

/// Paused lives at offset 20 in slot 1 (packed with ownerB).
pub fn paused_from_slot1(word: B256) -> bool {
    let n = U256::from_be_bytes(word.0);
    ((n >> 160) & U256::from(1u64)) == U256::from(1u64)
}

/// PaymentToken balanceOf mapping base (inspect: slot 1).
#[allow(dead_code)]
pub const PAYMENT_BALANCE_OF_BASE: u64 = 1;

#[allow(dead_code)]
pub fn balance_of_slot(account: Address) -> B256 {
    let mut buf = [0u8; 64];
    buf[12..32].copy_from_slice(account.as_slice());
    buf[56..64].copy_from_slice(&PAYMENT_BALANCE_OF_BASE.to_be_bytes());
    keccak256(buf)
}

/// PaymasterEntry feeTokenAllowlisted mapping base (inspect: slot 2).
#[allow(dead_code)]
pub const PAYMASTER_FEE_TOKEN_BASE: u64 = 2;

#[allow(dead_code)]
pub fn fee_token_allowlisted_slot(token: Address) -> B256 {
    let mut buf = [0u8; 64];
    buf[12..32].copy_from_slice(token.as_slice());
    buf[56..64].copy_from_slice(&PAYMASTER_FEE_TOKEN_BASE.to_be_bytes());
    keccak256(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    #[test]
    fn frozen_slot_matches_foundry_formula() {
        // Same formula as Sprint0 test_frozenSlotKey_matchesLayout / stdstore.
        let who = address!("0x00000000000000000000000000000000000A71CE");
        let slot = frozen_slot(who);
        // Hand-check: keccak256(abi.encode(who, 4))
        let mut buf = [0u8; 64];
        buf[12..32].copy_from_slice(who.as_slice());
        buf[63] = 4;
        assert_eq!(slot, keccak256(buf));
    }

    #[test]
    fn paused_bit_at_160() {
        let n = U256::from(1u64) << 160;
        let word = B256::from(n);
        assert!(paused_from_slot1(word));
        assert!(!paused_from_slot1(B256::ZERO));
    }
}
