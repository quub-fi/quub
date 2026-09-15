//! F201 Policy, F202 ISO memo, F203 Paymaster. No `reth-*` dependency.

pub mod abi;
pub mod error;
pub mod iso_memo;
pub mod paymaster;
pub mod policy;

pub use error::PrecompileError;

/// Reconstruct an Alloy-0.8 address from 20 bytes (used by `quub-evm`).
pub fn address_from_slice(bytes: &[u8]) -> alloy_primitives::Address {
    alloy_primitives::Address::from_slice(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, Address, B256, U256};
    use alloy_sol_types::{SolCall, SolValue};
    use quub_iso::validate_and_commit;
    use quub_policy::{check as policy_check, PolicyState};
    use quub_primitives::{
        Memo, MsgType, PolicyCheck, PolicyReason, CCY_USD, FEE_TOKEN_DEVNET, PAYMENT_TOKEN,
        QUUB_ISO_MEMO, QUUB_PAYMASTER, QUUB_POLICY,
    };

    #[test]
    fn addresses_are_f201_f202_f203() {
        assert_eq!(policy::ADDRESS, QUUB_POLICY);
        assert_eq!(iso_memo::ADDRESS, QUUB_ISO_MEMO);
        assert_eq!(paymaster::ADDRESS, QUUB_PAYMASTER);
        assert_eq!(
            format!("{:?}", policy::ADDRESS).to_lowercase(),
            "0x000000000000000000000000000000000000f201"
        );
        assert_eq!(
            format!("{:?}", iso_memo::ADDRESS).to_lowercase(),
            "0x000000000000000000000000000000000000f202"
        );
        assert_eq!(
            format!("{:?}", paymaster::ADDRESS).to_lowercase(),
            "0x000000000000000000000000000000000000f203"
        );
    }

    #[test]
    fn policy_unknown_selector_reverts() {
        let state = PolicyState::new();
        let err = policy::run(&[0xde, 0xad, 0xbe, 0xef], 10_000, PAYMENT_TOKEN, &state, 0)
            .unwrap_err();
        assert_eq!(err, PrecompileError::empty_revert());
    }

    #[test]
    fn policy_non_token_caller_reverts() {
        // Real bytes32 zero (32 zero bytes). Not a truncated `0x0` hex literal.
        let tr_hash = B256::from([0u8; 32]);
        assert_eq!(
            format!("{tr_hash}"),
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        let state = PolicyState::new();
        let eoa = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        let calldata = abi::CheckCall {
            token: PAYMENT_TOKEN,
            from: eoa,
            to: address!("0x70997970C51812dc3A010C7d01b50e0d17dc79C8"),
            amount: U256::from(1u64),
            trHash: tr_hash,
        }
        .abi_encode();
        // EOA / non-F210 caller must empty-revert (only PAYMENT_TOKEN may call F201).
        let err = policy::run(&calldata, 10_000, eoa, &state, 0).unwrap_err();
        assert_eq!(err, PrecompileError::empty_revert());
        let err_zero = policy::run(&calldata, 10_000, Address::ZERO, &state, 0).unwrap_err();
        assert_eq!(err_zero, PrecompileError::empty_revert());
    }

    #[test]
    fn policy_reason_codes_match_quub() {
        let mut state = PolicyState::new();
        let from = address!("0x0000000000000000000000000000000000000001");
        let to = address!("0x0000000000000000000000000000000000000002");
        let token = PAYMENT_TOKEN;
        state.freeze(from);

        let req = PolicyCheck {
            token,
            from,
            to,
            amount: U256::from(100u64),
            tr_hash: B256::ZERO,
        };
        let (allowed, reason) = policy_check(&state, &req);
        assert!(!allowed);
        assert_eq!(reason, PolicyReason::FrozenFrom);

        let calldata = abi::CheckCall {
            token,
            from,
            to,
            amount: req.amount,
            trHash: req.tr_hash,
        }
        .abi_encode();
        let (out, gas) =
            policy::run(&calldata, 20_000, PAYMENT_TOKEN, &state, 0).expect("run");
        assert_eq!(gas, policy::BASE_GAS);
        let (got_ok, got_reason): (bool, u16) = <(bool, u16)>::abi_decode(&out, false).unwrap();
        assert_eq!(got_ok, allowed);
        assert_eq!(got_reason, reason.as_u16());
    }

    #[test]
    fn iso_hash_matches_quub_iso() {
        let origin = address!("0x00000000000000000000000000000000000000AA");
        let memo = Memo::new(
            alloy_primitives::fixed_bytes!(
                "1111111111111111111111111111111111111111111111111111111111111111"
            ),
            alloy_primitives::fixed_bytes!("22222222222222222222222222222222"),
            alloy_primitives::fixed_bytes!(
                "3333333333333333333333333333333333333333333333333333333333333333"
            ),
            CCY_USD,
            MsgType::Pacs008,
        );
        let expected = validate_and_commit(&memo, origin).unwrap();
        let calldata = abi::ValidateAndCommitCall {
            endToEndId: memo.end_to_end_id,
            uetr: memo.uetr,
            instrId: memo.instr_id,
            ccy: memo.ccy,
            msgType: memo.msg_type.as_u8(),
        }
        .abi_encode();
        let (out, gas) = iso_memo::run(&calldata, 10_000, origin).expect("run");
        assert_eq!(gas, iso_memo::GAS);
        let got = B256::abi_decode(&out, false).unwrap();
        assert_eq!(got, expected);
    }

    #[test]
    fn paymaster_unlisted_token_errors() {
        let unlisted = address!("0x00000000000000000000000000000000000000FF");
        let calldata = abi::QuoteCall {
            feeToken: unlisted,
            gasLimit: U256::from(21_000u64),
            gasPrice: U256::from(1u64),
        }
        .abi_encode();
        let err = paymaster::run(&calldata, 10_000, Address::ZERO).unwrap_err();
        assert_eq!(err, PrecompileError::empty_revert());

        let listed = abi::QuoteCall {
            feeToken: FEE_TOKEN_DEVNET,
            gasLimit: U256::from(21_000u64),
            gasPrice: U256::from(1u64),
        }
        .abi_encode();
        let (out, _) = paymaster::run(&listed, 10_000, Address::ZERO).expect("listed quote");
        let amount = U256::abi_decode(&out, false).unwrap();
        assert_eq!(amount, U256::from(21_000u64));
    }
}
