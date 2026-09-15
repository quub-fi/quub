//! Payment-lane classifier (Sprint 3 / ADR-017).
//!
//! Payment iff `to == F210` and calldata is exactly `transferWithMemo` (selector + length).
//! Plain `transfer` / `transferFrom` on F210 are **general**. Prefix-only matching is forbidden.

use alloy_primitives::Address;
use quub_primitives::{
    PAYMENT_TOKEN, TRANSFER_WITH_MEMO_CALLDATA_LEN, TRANSFER_WITH_MEMO_SELECTOR,
};

/// Returns true when `to` is F210 and `input` is exact-length `transferWithMemo`.
pub fn is_payment(to: Address, input: &[u8]) -> bool {
    is_payment_bytes(to.as_slice(), input)
}

/// Version-agnostic classifier for Reth (alloy 1.x) and services (alloy 0.8).
pub fn is_payment_bytes(to: &[u8], input: &[u8]) -> bool {
    if to != PAYMENT_TOKEN.as_slice() {
        return false;
    }
    if input.len() != TRANSFER_WITH_MEMO_CALLDATA_LEN {
        return false;
    }
    input[..4] == TRANSFER_WITH_MEMO_SELECTOR
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;
    use quub_primitives::{TRANSFER_CALLDATA_LEN, TRANSFER_SELECTOR};

    fn calldata(sel: [u8; 4], total_len: usize) -> Vec<u8> {
        let mut v = sel.to_vec();
        v.resize(total_len, 0);
        v
    }

    #[test]
    fn memo_exact_on_f210_is_payment() {
        let input = calldata(TRANSFER_WITH_MEMO_SELECTOR, TRANSFER_WITH_MEMO_CALLDATA_LEN);
        assert!(is_payment(PAYMENT_TOKEN, &input));
    }

    #[test]
    fn memo_plus_junk_is_not() {
        let input = calldata(TRANSFER_WITH_MEMO_SELECTOR, TRANSFER_WITH_MEMO_CALLDATA_LEN + 1);
        assert!(!is_payment(PAYMENT_TOKEN, &input));
    }

    #[test]
    fn plain_transfer_on_f210_is_general() {
        let input = calldata(TRANSFER_SELECTOR, TRANSFER_CALLDATA_LEN);
        assert!(!is_payment(PAYMENT_TOKEN, &input));
    }

    #[test]
    fn wrong_to_is_not() {
        let input = calldata(TRANSFER_WITH_MEMO_SELECTOR, TRANSFER_WITH_MEMO_CALLDATA_LEN);
        let other = address!("0x00000000000000000000000000000000000000aa");
        assert!(!is_payment(other, &input));
    }

    #[test]
    fn bad_selector_is_not() {
        let input = calldata([0xde, 0xad, 0xbe, 0xef], TRANSFER_WITH_MEMO_CALLDATA_LEN);
        assert!(!is_payment(PAYMENT_TOKEN, &input));
    }

    #[test]
    fn transfer_with_memo_selector_matches_solidity() {
        let sig = b"transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)";
        let h = alloy_primitives::keccak256(sig);
        assert_eq!(&TRANSFER_WITH_MEMO_SELECTOR, &h[..4]);
        assert_eq!(TRANSFER_WITH_MEMO_CALLDATA_LEN, 4 + 32 * 9);
    }
}
