//! Precompile set: Eth builtins plus F201–F203 when spec >= Prague.

use crate::wrap;
use alloy_primitives::{address, Address};
use reth_ethereum::evm::revm::{
    handler::EthPrecompiles,
    precompile::{Precompile, PrecompileId, Precompiles},
    primitives::hardfork::SpecId,
};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

const F201: Address = address!("0x000000000000000000000000000000000000F201");
const F202: Address = address!("0x000000000000000000000000000000000000F202");
const F203: Address = address!("0x000000000000000000000000000000000000F203");
const ECRECOVER: Address = address!("0x0000000000000000000000000000000000000001");

fn extend_quub(set: &mut Precompiles) {
    set.extend([
        Precompile::new(PrecompileId::custom("quub-policy"), F201, wrap::policy),
        Precompile::new(PrecompileId::custom("quub-iso-memo"), F202, wrap::iso_memo),
        Precompile::new(PrecompileId::custom("quub-paymaster"), F203, wrap::paymaster),
    ]);
}

/// Builtins for `spec`, plus Quub precompiles when `spec >= PRAGUE`.
/// Cached per spec so Osaka does not reuse a Prague-only map.
pub fn precompiles_for_spec(spec: SpecId) -> &'static Precompiles {
    static CACHE: OnceLock<Mutex<HashMap<SpecId, &'static Precompiles>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock().expect("precompile cache");
    *cache.entry(spec).or_insert_with(|| {
        let mut set = (*EthPrecompiles::new(spec).precompiles).clone();
        if spec >= SpecId::PRAGUE {
            extend_quub(&mut set);
        }
        Box::leak(Box::new(set))
    })
}

pub fn contains_quub_and_ecrecover(set: &Precompiles) -> bool {
    set.contains(&F201) && set.contains(&F202) && set.contains(&F203) && set.contains(&ECRECOVER)
}
