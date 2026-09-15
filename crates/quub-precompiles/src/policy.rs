//! F201 Policy — `check`. Calls `xzero_policy::check`.

use crate::abi::CheckCall;
use crate::error::PrecompileError;
use alloy_primitives::{Address, Bytes};
use alloy_sol_types::{SolCall, SolValue};
use quub_primitives::{PolicyCheck, QUUB_POLICY};
use std::sync::{Mutex, OnceLock};
use xzero_policy::{check as xzero_check, PolicyState};

pub const ADDRESS: Address = QUUB_POLICY;
pub const BASE_GAS: u64 = 3_000;
pub const TR_HASH_GAS: u64 = 2_000;

fn state() -> &'static Mutex<PolicyState> {
    static STATE: OnceLock<Mutex<PolicyState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(PolicyState::new()))
}

/// Sprint 1 test hook. Production F201 will staticcall F211 (Sprint 1.5+).
pub fn replace_state(next: PolicyState) {
    *state().lock().expect("policy state") = next;
}

pub fn run(input: &[u8], gas: u64, _caller: Address) -> Result<(Bytes, u64), PrecompileError> {
    if input.len() < 4 {
        return Err(PrecompileError::empty_revert());
    }
    if input[..4] != CheckCall::SELECTOR {
        return Err(PrecompileError::empty_revert());
    }

    let decoded = CheckCall::abi_decode(input).map_err(|_| PrecompileError::empty_revert())?;
    let needed = if decoded.trHash != Default::default() {
        BASE_GAS + TR_HASH_GAS
    } else {
        BASE_GAS
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
    let guard = state().lock().expect("policy state");
    let (allowed, reason) = xzero_check(&guard, &req);
    let out = (allowed, reason.as_u16()).abi_encode();
    Ok((Bytes::from(out), needed))
}
