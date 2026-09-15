//! Precompile set: Eth builtins plus F201–F203 when spec >= Prague.
//!
//! F201 and F202 use [`DynPrecompile::new_stateful`] — cache key is calldata only;
//! F201 reads F211 storage and F202 includes tx.origin in the hash.

use crate::wrap;
use alloy_evm::precompiles::{DynPrecompile, PrecompilesMap};
use alloy_primitives::{address, Address};
use reth_ethereum::evm::revm::{
    handler::EthPrecompiles,
    precompile::PrecompileId,
    primitives::hardfork::SpecId,
};

const F201: Address = address!("0x000000000000000000000000000000000000F201");
const F202: Address = address!("0x000000000000000000000000000000000000F202");
const F203: Address = address!("0x000000000000000000000000000000000000F203");
#[cfg(test)]
const ECRECOVER: Address = address!("0x0000000000000000000000000000000000000001");

fn inject_quub(map: &mut PrecompilesMap) {
    map.apply_precompile(&F201, |_| {
        Some(DynPrecompile::new_stateful(
            PrecompileId::custom("quub-policy"),
            wrap::policy,
        ))
    });
    map.apply_precompile(&F202, |_| {
        Some(DynPrecompile::new_stateful(
            PrecompileId::custom("quub-iso-memo"),
            wrap::iso_memo,
        ))
    });
    map.apply_precompile(&F203, |_| {
        Some(DynPrecompile::new(
            PrecompileId::custom("quub-paymaster"),
            wrap::paymaster,
        ))
    });
}

/// Inject F201–F203 into an existing precompile map (Eth or OP).
pub fn inject_quub_precompiles(map: &mut PrecompilesMap) {
    inject_quub(map);
}

/// Builtins for `spec`, plus Quub precompiles when `spec >= PRAGUE`.
pub fn precompiles_map_for_spec(spec: SpecId) -> PrecompilesMap {
    let mut map = PrecompilesMap::from_static(EthPrecompiles::new(spec).precompiles);
    if spec >= SpecId::PRAGUE {
        inject_quub(&mut map);
    }
    map
}

#[cfg(test)]
pub(crate) fn contains_quub_and_ecrecover(map: &PrecompilesMap) -> bool {
    map.get(&F201).is_some()
        && map.get(&F202).is_some()
        && map.get(&F203).is_some()
        && map.get(&ECRECOVER).is_some()
}
