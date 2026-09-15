//! F202 ISO memo — `validateAndCommit`. Calls `xzero_iso::validate_and_commit`.

use crate::abi::ValidateAndCommitCall;
use crate::error::PrecompileError;
use alloy_primitives::{Address, Bytes};
use alloy_sol_types::{SolCall, SolValue};
use quub_primitives::{Memo, MsgType, QUUB_ISO_MEMO};
use xzero_iso::validate_and_commit;

pub const ADDRESS: Address = QUUB_ISO_MEMO;
pub const GAS: u64 = 2_500;

pub fn run(input: &[u8], gas: u64, caller: Address) -> Result<(Bytes, u64), PrecompileError> {
    if input.len() < 4 {
        return Err(PrecompileError::empty_revert());
    }
    if input[..4] != ValidateAndCommitCall::SELECTOR {
        return Err(PrecompileError::empty_revert());
    }
    if gas < GAS {
        return Err(PrecompileError::OutOfGas);
    }

    let decoded =
        ValidateAndCommitCall::abi_decode(input, false).map_err(|_| PrecompileError::empty_revert())?;
    let msg_type =
        MsgType::from_u8(decoded.msgType).ok_or_else(PrecompileError::empty_revert)?;
    let memo = Memo::new(
        decoded.endToEndId,
        decoded.uetr,
        decoded.instrId,
        decoded.ccy,
        msg_type,
    );
    let hash = validate_and_commit(&memo, caller).map_err(|_| PrecompileError::empty_revert())?;
    Ok((Bytes::from(hash.abi_encode()), GAS))
}
