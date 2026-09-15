//! Quub EVM: register F201–F203 on Ethereum builtins. No consensus types.

mod factory;
mod policy_load;
mod precompiles;
mod slots;
mod wrap;

pub use factory::{QuubEvmFactory, QuubExecutorBuilder};
pub use precompiles::precompiles_map_for_spec;

#[cfg(test)]
mod tests {
    use super::precompiles::{contains_quub_and_ecrecover, precompiles_map_for_spec};
    use crate::factory::QuubEvmFactory;
    use crate::slots::{frozen_slot, paused_from_slot1, FROZEN_MAPPING_BASE};
    use alloy_primitives::{address, keccak256, B256, U256};
    use reth_ethereum::evm::revm::primitives::hardfork::SpecId;

    #[test]
    fn map_contains_f201_f202_f203_and_ecrecover() {
        let prague = precompiles_map_for_spec(SpecId::PRAGUE);
        assert!(contains_quub_and_ecrecover(&prague));

        for spec in [SpecId::PRAGUE, SpecId::OSAKA] {
            let map = precompiles_map_for_spec(spec);
            assert!(
                contains_quub_and_ecrecover(&map),
                "missing Quub precompiles on {spec:?}"
            );
        }
    }

    /// Sprint 2: factory still registers F201–F203 on `spec >= PRAGUE` (Mode A blocked elsewhere).
    #[test]
    fn mode_a_factory_builds() {
        let _ = QuubEvmFactory;
        for spec in [SpecId::PRAGUE, SpecId::OSAKA] {
            let map = precompiles_map_for_spec(spec);
            assert!(
                contains_quub_and_ecrecover(&map),
                "QuubEvmFactory precompile set missing on {spec:?}"
            );
        }
    }

    #[test]
    fn policy_reads_admin_freeze() {
        let from = address!("0x00000000000000000000000000000000000A71CE");
        let state = quub_precompiles::policy::state_from_admin(
            false,
            None,
            Some(from.into_array()),
            None,
        );
        let from_q = quub_precompiles::address_from_slice(from.as_slice());
        let to_q = quub_precompiles::address_from_slice(
            address!("0x00000000000000000000000000000000000000B0").as_slice(),
        );
        let mut amount = [0u8; 32];
        amount[31] = 100;
        let state_amt = quub_precompiles::policy::state_from_admin(false, Some(amount), None, None);
        let amount = state_amt.amount_limit.expect("amount");
        let req = quub_primitives::PolicyCheck {
            token: quub_primitives::PAYMENT_TOKEN,
            from: from_q,
            to: to_q,
            amount,
            tr_hash: Default::default(),
        };
        let (ok, reason) = quub_policy::check(&state, &req);
        assert!(!ok);
        assert_eq!(reason, quub_primitives::PolicyReason::FrozenFrom);

        let mut buf = [0u8; 64];
        buf[12..32].copy_from_slice(from.as_slice());
        buf[63] = FROZEN_MAPPING_BASE as u8;
        assert_eq!(frozen_slot(from), keccak256(buf));
        assert!(paused_from_slot1(B256::from(U256::from(1u64) << 160)));
    }
}
