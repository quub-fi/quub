//! Quub EVM: register F201–F203 on Ethereum builtins. No consensus types.

mod factory;
mod precompiles;
mod wrap;

pub use factory::{QuubEvmFactory, QuubExecutorBuilder};
pub use precompiles::precompiles_map_for_spec;

#[cfg(test)]
mod tests {
    use super::precompiles::{contains_quub_and_ecrecover, precompiles_map_for_spec};
    use reth_ethereum::evm::revm::primitives::hardfork::SpecId;

    #[test]
    fn map_contains_f201_f202_f203_and_ecrecover() {
        let prague = precompiles_map_for_spec(SpecId::PRAGUE);
        assert!(contains_quub_and_ecrecover(&prague));

        // Later specs must keep Quub precompiles (not only exact Prague).
        for spec in [SpecId::PRAGUE, SpecId::OSAKA] {
            let map = precompiles_map_for_spec(spec);
            assert!(
                contains_quub_and_ecrecover(&map),
                "missing Quub precompiles on {spec:?}"
            );
        }
    }
}
