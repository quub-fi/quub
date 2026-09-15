//! Wrap Alloy 0.8 `run` into reth/revm precompile closures. Conversion only here.

use alloy_primitives::{Address, Bytes};
use quub_precompiles::{address_from_slice, iso_memo, paymaster, policy, PrecompileError as QuubErr};
use reth_ethereum::evm::revm::precompile::{PrecompileError, PrecompileOutput};

fn map_out(
    result: Result<(impl AsRef<[u8]>, u64), QuubErr>,
) -> Result<PrecompileOutput, PrecompileError> {
    match result {
        Ok((bytes, used)) => Ok(PrecompileOutput {
            gas_used: used,
            bytes: Bytes::copy_from_slice(bytes.as_ref()),
            reverted: false,
        }),
        Err(QuubErr::OutOfGas) => Err(PrecompileError::OutOfGas),
        Err(QuubErr::Revert(_)) => Err(PrecompileError::other("revert")),
    }
}

pub fn policy(input: &[u8], gas: u64, caller: Address) -> Result<PrecompileOutput, PrecompileError> {
    map_out(policy::run(input, gas, address_from_slice(caller.as_slice())))
}

pub fn iso_memo(
    input: &[u8],
    gas: u64,
    caller: Address,
) -> Result<PrecompileOutput, PrecompileError> {
    map_out(iso_memo::run(input, gas, address_from_slice(caller.as_slice())))
}

pub fn paymaster(
    input: &[u8],
    gas: u64,
    caller: Address,
) -> Result<PrecompileOutput, PrecompileError> {
    map_out(paymaster::run(input, gas, address_from_slice(caller.as_slice())))
}
