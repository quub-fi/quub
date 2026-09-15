//! Wrap Alloy DynPrecompile closures. F201 loads F211 via sload; F202 uses tx.origin.

use crate::policy_load;
use alloy_evm::precompiles::PrecompileInput;
use alloy_primitives::Bytes;
use quub_precompiles::{
    address_from_slice, iso_memo, paymaster, policy, PrecompileError as QuubErr,
};
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

pub fn policy(mut input: PrecompileInput<'_>) -> PrecompileResult {
    let caller = address_from_slice(input.caller.as_slice());
    let reservoir = input.reservoir;
    let gas = input.gas;
    let data = input.data;

    let (state, storage_gas) = match policy_load::load_policy_state_for(input.internals_mut(), data)
    {
        Ok(v) => v,
        Err(e) => return map_out(Err::<(Bytes, u64), _>(e), reservoir),
    };

    map_out(
        policy::run(data, gas, caller, &state, storage_gas),
        reservoir,
    )
}

pub fn iso_memo(input: PrecompileInput<'_>) -> PrecompileResult {
    // Origin is in the memo hash preimage — never use input.caller (F210 when token calls).
    let origin = address_from_slice(input.internals.tx_origin().as_slice());
    map_out(iso_memo::run(input.data, input.gas, origin), input.reservoir)
}

pub fn paymaster(input: PrecompileInput<'_>) -> PrecompileResult {
    let caller = address_from_slice(input.caller.as_slice());
    map_out(paymaster::run(input.data, input.gas, caller), input.reservoir)
}
