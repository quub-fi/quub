//! F201 Policy — `check`. Calls `quub_policy::check` on a caller-supplied state.
//!
//! Production loads state from F211 storage in `quub-evm` (sload). This crate stays DB-free.

use crate::abi::CheckCall;
use crate::error::PrecompileError;
use alloy_primitives::{Address, Bytes, B256, U256};
use alloy_sol_types::{SolCall, SolValue};
use quub_policy::{check as policy_check, PolicyState};
use quub_primitives::{PolicyCheck, PAYMENT_TOKEN, QUUB_POLICY};

pub const ADDRESS: Address = QUUB_POLICY;
pub const BASE_GAS: u64 = 3_000;
pub const TR_HASH_GAS: u64 = 2_000;
/// Cold SLOAD cost charged by quub-evm per PolicyAdmin read (Prague-class).
pub const COLD_SLOAD_GAS: u64 = 2_100;

/// Build a [`PolicyState`] from raw admin bits (avoids alloy version mixing in quub-evm).
pub fn state_from_admin(
    paused: bool,
    threshold_be: Option<[u8; 32]>,
    frozen_from: Option<[u8; 20]>,
    frozen_to: Option<[u8; 20]>,
) -> PolicyState {
    let mut state = PolicyState::new();
    state.paused = paused;
    if let Some(be) = threshold_be {
        state.amount_limit = Some(U256::from_be_bytes(be));
    }
    if let Some(b) = frozen_from {
        state.freeze(Address::from_slice(&b));
    }
    if let Some(b) = frozen_to {
        state.freeze(Address::from_slice(&b));
    }
    state
}

pub fn run(
    input: &[u8],
    gas: u64,
    caller: Address,
    state: &PolicyState,
    storage_gas: u64,
) -> Result<(Bytes, u64), PrecompileError> {
    if input.len() < 4 {
        return Err(PrecompileError::empty_revert());
    }
    if input[..4] != CheckCall::SELECTOR {
        return Err(PrecompileError::empty_revert());
    }
    // Only PAYMENT_TOKEN (F210) may call F201.
    if caller != PAYMENT_TOKEN {
        return Err(PrecompileError::empty_revert());
    }

    let decoded = CheckCall::abi_decode(input, false).map_err(|_| PrecompileError::empty_revert())?;
    let needed = if decoded.trHash != B256::ZERO {
        BASE_GAS + TR_HASH_GAS + storage_gas
    } else {
        BASE_GAS + storage_gas
    };
    if gas < needed {
        return Err(PrecompileError::OutOfGas);
    }

    let req = PolicyCheck {
        token: decoded.token,
        from: decoded.from,
        to: decoded.to,
        amount: decoded.amount,
        tr_hash: decoded.trHash,
    };
    let (allowed, reason) = policy_check(state, &req);
    let out = (allowed, reason.as_u16()).abi_encode();
    Ok((Bytes::from(out), needed))
}
