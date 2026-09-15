//! Payment-lane classifier. Prefix-only matching is forbidden.
//!
//! Lengths come from `quub-primitives`, measured against PaymentToken.sol.

use alloy_primitives::Address;
use quub_primitives::{
    TRANSFER_CALLDATA_LEN, TRANSFER_FROM_CALLDATA_LEN, TRANSFER_FROM_SELECTOR, TRANSFER_SELECTOR,
    TRANSFER_WITH_MEMO_CALLDATA_LEN, TRANSFER_WITH_MEMO_SELECTOR,
};
use std::collections::HashSet;

pub fn is_payment(to: Address, input: &[u8], registered: &HashSet<Address>) -> bool {
    if !registered.contains(&to) {
        return false;
    }
    if input.len() < 4 {
        return false;
    }
    let sel: [u8; 4] = input[..4].try_into().expect("len >= 4");
    let len = input.len();
    match sel {
        TRANSFER_SELECTOR => len == TRANSFER_CALLDATA_LEN,
        TRANSFER_FROM_SELECTOR => len == TRANSFER_FROM_CALLDATA_LEN,
        TRANSFER_WITH_MEMO_SELECTOR => len == TRANSFER_WITH_MEMO_CALLDATA_LEN,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;
    use quub_primitives::PAYMENT_TOKEN;

    fn registered() -> HashSet<Address> {
        let mut s = HashSet::new();
        s.insert(PAYMENT_TOKEN);
        s
    }

    fn calldata(sel: [u8; 4], extra: usize) -> Vec<u8> {
        let mut v = sel.to_vec();
        v.extend(std::iter::repeat(0u8).take(extra));
        v
    }

    #[test]
    fn transfer_exact_len_is_payment() {
        let input = calldata(TRANSFER_SELECTOR, TRANSFER_CALLDATA_LEN - 4);
        assert_eq!(input.len(), TRANSFER_CALLDATA_LEN);
        assert!(is_payment(PAYMENT_TOKEN, &input, &registered()));
    }

    #[test]
    fn transfer_prefix_plus_junk_is_not() {
        let input = calldata(TRANSFER_SELECTOR, TRANSFER_CALLDATA_LEN - 4 + 1);
        assert!(!is_payment(PAYMENT_TOKEN, &input, &registered()));
    }

    #[test]
    fn unregistered_token_is_not() {
        let input = calldata(TRANSFER_SELECTOR, TRANSFER_CALLDATA_LEN - 4);
        let other = address!("0x00000000000000000000000000000000000000aa");
        assert!(!is_payment(other, &input, &registered()));
    }

    #[test]
    fn transfer_with_memo_len_matches_solidity() {
        let sig = b"transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)";
        let h = alloy_primitives::keccak256(sig);
        assert_eq!(&TRANSFER_WITH_MEMO_SELECTOR, &h[..4]);
        // selector + 9 ABI words
        assert_eq!(TRANSFER_WITH_MEMO_CALLDATA_LEN, 4 + 32 * 9);
        let input = calldata(TRANSFER_WITH_MEMO_SELECTOR, TRANSFER_WITH_MEMO_CALLDATA_LEN - 4);
        assert!(is_payment(PAYMENT_TOKEN, &input, &registered()));
        let junk = calldata(TRANSFER_WITH_MEMO_SELECTOR, TRANSFER_WITH_MEMO_CALLDATA_LEN - 4 + 1);
        assert!(!is_payment(PAYMENT_TOKEN, &junk, &registered()));
    }
}
