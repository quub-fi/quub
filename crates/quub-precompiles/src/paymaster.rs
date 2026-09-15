//! F203 Paymaster hook. Year-1 rate is a 1e6-scale constant, not an oracle.

use crate::abi::{QuoteCall, TakeFeeCall};
use crate::error::PrecompileError;
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{SolCall, SolValue};
use quub_primitives::{FEE_TOKEN_DEVNET, PAYMASTER_TEST_HOST, QUUB_PAYMASTER};

pub const ADDRESS: Address = QUUB_PAYMASTER;
pub const GAS: u64 = 8_000;
/// Same scale as contracts/src/PaymasterEntry.sol `RATE_1E6`.
pub const RATE_1E6: u64 = 1_000_000;

pub fn quote_amount(fee_token: Address, gas_limit: U256, gas_price: U256) -> Result<U256, PrecompileError> {
    if fee_token != FEE_TOKEN_DEVNET {
        return Err(PrecompileError::empty_revert());
    }
    Ok(gas_limit
        .checked_mul(gas_price)
        .and_then(|v| v.checked_mul(U256::from(RATE_1E6)))
        .and_then(|v| v.checked_div(U256::from(RATE_1E6)))
        .ok_or_else(PrecompileError::empty_revert)?)
}

pub fn run(input: &[u8], gas: u64, caller: Address) -> Result<(Bytes, u64), PrecompileError> {
    if input.len() < 4 {
        return Err(PrecompileError::empty_revert());
    }
    if gas < GAS {
        return Err(PrecompileError::OutOfGas);
    }

    if input[..4] == QuoteCall::SELECTOR {
        let decoded = QuoteCall::abi_decode(input).map_err(|_| PrecompileError::empty_revert())?;
        let amount = quote_amount(decoded.feeToken, decoded.gasLimit, decoded.gasPrice)?;
        return Ok((Bytes::from(amount.abi_encode()), GAS));
    }

    if input[..4] == TakeFeeCall::SELECTOR {
        if caller != PAYMASTER_TEST_HOST {
            return Err(PrecompileError::empty_revert());
        }
        let decoded =
            TakeFeeCall::abi_decode(input).map_err(|_| PrecompileError::empty_revert())?;
        if decoded.feeToken != FEE_TOKEN_DEVNET || decoded.tokenAmount.is_zero() {
            return Err(PrecompileError::empty_revert());
        }
        // Stateful debit waits until F213 is on the node (Sprint 1.5).
        return Ok((Bytes::new(), GAS));
    }

    Err(PrecompileError::empty_revert())
}
