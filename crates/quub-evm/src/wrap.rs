//! Wrap Alloy 0.8 `run` into alloy-evm `DynPrecompile` closures.
//! Address / Bytes conversion lives only here.

use alloy_evm::precompiles::PrecompileInput;
use alloy_primitives::Bytes;
use quub_precompiles::{address_from_slice, iso_memo, paymaster, policy, PrecompileError as QuubErr};
use reth_ethereum::evm::revm::precompile::{PrecompileHalt, PrecompileOutput, PrecompileResult};

fn map_out(
    result: Result<(impl AsRef<[u8]>, u64), QuubErr>,
    reservoir: u64,
) -> PrecompileResult {
    match result {
        Ok((bytes, used)) => Ok(PrecompileOutput::new(
            used,
            Bytes::copy_from_slice(bytes.as_ref()),
            reservoir,
        )),
        Err(QuubErr::OutOfGas) => Ok(PrecompileOutput::halt(PrecompileHalt::OutOfGas, reservoir)),
        Err(QuubErr::Revert(data)) => Ok(PrecompileOutput::revert(
            0,
            Bytes::copy_from_slice(data.as_ref()),
            reservoir,
        )),
    }
}

pub fn policy(input: PrecompileInput<'_>) -> PrecompileResult {
    let caller = address_from_slice(input.caller.as_slice());
    map_out(policy::run(input.data, input.gas, caller), input.reservoir)
}

pub fn iso_memo(input: PrecompileInput<'_>) -> PrecompileResult {
    let caller = address_from_slice(input.caller.as_slice());
    map_out(iso_memo::run(input.data, input.gas, caller), input.reservoir)
}

pub fn paymaster(input: PrecompileInput<'_>) -> PrecompileResult {
    let caller = address_from_slice(input.caller.as_slice());
    map_out(paymaster::run(input.data, input.gas, caller), input.reservoir)
}
