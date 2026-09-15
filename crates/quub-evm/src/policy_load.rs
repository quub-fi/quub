//! Load PolicyState from on-chain PolicyAdmin (F211) storage.

use crate::slots::{frozen_slot, paused_from_slot1, OWNER_B_PAUSED_SLOT, THRESHOLD_SLOT};
use alloy_evm::EvmInternals;
use alloy_primitives::{address, Address, B256, U256};
use quub_precompiles::{policy, PrecompileError};
use quub_policy::PolicyState;
use reth_ethereum::evm::revm::primitives::KECCAK_EMPTY;

const F211: Address = address!("0x000000000000000000000000000000000000F211");

/// `check(address,address,address,uint256,bytes32)` selector.
const CHECK_SELECTOR: [u8; 4] = [0x59, 0x84, 0x27, 0x55];

/// Fail-closed state when F211 has no code.
pub fn deny_closed_state() -> PolicyState {
    let mut s = PolicyState::new();
    s.paused = true;
    s
}

fn decode_from_to(data: &[u8]) -> Result<(Address, Address), PrecompileError> {
    if data.len() < 4 + 32 * 5 {
        return Err(PrecompileError::empty_revert());
    }
    if data[..4] != CHECK_SELECTOR {
        // Still allow unknown to be handled by policy::run; but we need from/to.
        // If selector wrong, policy::run will empty-revert — return zeros so load is cheap.
        return Err(PrecompileError::empty_revert());
    }
    // ABI: word1 token, word2 from, word3 to (address in last 20 bytes of each word)
    let from = Address::from_slice(&data[4 + 32 + 12..4 + 32 + 32]);
    let to = Address::from_slice(&data[4 + 64 + 12..4 + 64 + 32]);
    Ok((from, to))
}

pub fn load_policy_state_for(
    internals: &mut EvmInternals<'_>,
    calldata: &[u8],
) -> Result<(PolicyState, u64), PrecompileError> {
    let mut reads: u64 = 0;

    let loaded = internals
        .load_account(F211)
        .map_err(|_| PrecompileError::empty_revert())?;
    reads += 1;
    if loaded.data.info.code_hash == KECCAK_EMPTY {
        return Ok((deny_closed_state(), policy::COLD_SLOAD_GAS * reads));
    }

    let (from, to) = decode_from_to(calldata)?;

    let paused_raw = internals
        .sload(F211, U256::from(OWNER_B_PAUSED_SLOT))
        .map_err(|_| PrecompileError::empty_revert())?
        .data;
    reads += 1;
    let paused = paused_from_slot1(B256::from(paused_raw.to_be_bytes::<32>()));

    let threshold = internals
        .sload(F211, U256::from(THRESHOLD_SLOT))
        .map_err(|_| PrecompileError::empty_revert())?
        .data;
    reads += 1;

    let frozen_from = internals
        .sload(F211, U256::from_be_bytes(frozen_slot(from).0))
        .map_err(|_| PrecompileError::empty_revert())?
        .data;
    reads += 1;

    let frozen_to = internals
        .sload(F211, U256::from_be_bytes(frozen_slot(to).0))
        .map_err(|_| PrecompileError::empty_revert())?
        .data;
    reads += 1;

    // Build PolicyState via quub-precompiles bridge (alloy 0.8 Address/U256).
    let state = policy::state_from_admin(
        paused,
        if threshold.is_zero() {
            None
        } else {
            Some(threshold.to_be_bytes::<32>())
        },
        if frozen_from.is_zero() {
            None
        } else {
            Some(from.into_array())
        },
        if frozen_to.is_zero() {
            None
        } else {
            Some(to.into_array())
        },
    );

    Ok((state, policy::COLD_SLOAD_GAS * reads))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_types::SolCall;
    use quub_precompiles::abi::CheckCall;

    #[test]
    fn check_selector_matches_abi() {
        assert_eq!(CHECK_SELECTOR, CheckCall::SELECTOR);
    }
}
