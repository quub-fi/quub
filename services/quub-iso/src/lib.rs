//! Quub Memo / Quub ISO — off-chain matching F202 `validateAndCommit` ABI.
//!
//! `memoHash = keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, origin))`
//! No `block.timestamp`. No names, IBANs, or XML.

use alloy_primitives::{Address, B256, FixedBytes, Keccak256};
use alloy_sol_types::{sol, SolValue};
use quub_primitives::{is_allowed_ccy, Memo, MsgType};
use thiserror::Error;

sol! {
    struct MemoEncode {
        bytes32 endToEndId;
        bytes16 uetr;
        bytes32 instrId;
        bytes3 ccy;
        uint8 msgType;
        address origin;
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IsoError {
    #[error("empty EndToEndId")]
    EmptyEndToEndId,
    #[error("missing UETR for msgType requiring it")]
    MissingUetr,
    #[error("currency not allowlisted")]
    BadCurrency,
    #[error("unknown msgType")]
    BadMsgType,
}

/// Validate memo fields and commit a stable hash (same inputs → same hash).
///
/// `origin` stands in for `tx.origin` / the submitting party off-chain.
pub fn validate_and_commit(memo: &Memo, origin: Address) -> Result<B256, IsoError> {
    if memo.end_to_end_id == B256::ZERO {
        return Err(IsoError::EmptyEndToEndId);
    }

    if memo.msg_type.requires_uetr() && memo.uetr == FixedBytes::<16>::ZERO {
        return Err(IsoError::MissingUetr);
    }

    if !is_allowed_ccy(memo.ccy) {
        return Err(IsoError::BadCurrency);
    }

    Ok(memo_hash(memo, origin))
}

/// `keccak256(abi.encode(...))` — no timestamp in the preimage.
pub fn memo_hash(memo: &Memo, origin: Address) -> B256 {
    let encoded = MemoEncode {
        endToEndId: memo.end_to_end_id,
        uetr: memo.uetr,
        instrId: memo.instr_id,
        ccy: memo.ccy,
        msgType: memo.msg_type.as_u8(),
        origin,
    }
    .abi_encode();

    let mut hasher = Keccak256::new();
    hasher.update(encoded);
    hasher.finalize()
}

/// Convenience wrapper taking raw ABI fields.
pub fn validate_and_commit_raw(
    end_to_end_id: B256,
    uetr: FixedBytes<16>,
    instr_id: B256,
    ccy: FixedBytes<3>,
    msg_type: u8,
    origin: Address,
) -> Result<B256, IsoError> {
    let msg_type = MsgType::from_u8(msg_type).ok_or(IsoError::BadMsgType)?;
    let memo = Memo::new(end_to_end_id, uetr, instr_id, ccy, msg_type);
    validate_and_commit(&memo, origin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, fixed_bytes};
    use quub_primitives::CCY_USD;

    fn sample_memo() -> Memo {
        Memo::new(
            fixed_bytes!("1111111111111111111111111111111111111111111111111111111111111111"),
            fixed_bytes!("22222222222222222222222222222222"),
            fixed_bytes!("3333333333333333333333333333333333333333333333333333333333333333"),
            CCY_USD,
            MsgType::Pacs008,
        )
    }

    #[test]
    fn empty_end_to_end_id_reverts() {
        let mut memo = sample_memo();
        memo.end_to_end_id = B256::ZERO;
        let origin = address!("0x00000000000000000000000000000000000000AA");
        assert_eq!(
            validate_and_commit(&memo, origin),
            Err(IsoError::EmptyEndToEndId)
        );
    }

    #[test]
    fn pacs008_without_uetr_reverts() {
        let mut memo = sample_memo();
        memo.uetr = FixedBytes::<16>::ZERO;
        let origin = address!("0x00000000000000000000000000000000000000AA");
        assert_eq!(
            validate_and_commit(&memo, origin),
            Err(IsoError::MissingUetr)
        );
    }

    #[test]
    fn usd_pacs008_stable_hash() {
        let memo = sample_memo();
        let origin = address!("0x00000000000000000000000000000000000000AA");
        let h1 = validate_and_commit(&memo, origin).expect("ok");
        let h2 = validate_and_commit(&memo, origin).expect("ok");
        assert_eq!(h1, h2);
        assert_ne!(h1, B256::ZERO);
    }
}
