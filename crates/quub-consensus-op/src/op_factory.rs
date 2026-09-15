//! Quub OP EVM factory: Optimism builtins + F201–F203.
//! Pattern: op-reth `node/tests/it/builder.rs` UniEvmFactory.

use alloy_evm::{
    Database, Evm, EvmEnv, EvmFactory,
    precompiles::PrecompilesMap,
    revm::{
        context::DBErrorMarker,
        inspector::{Inspector, NoOpInspector},
    },
};
use quub_evm::inject_quub_precompiles;
use reth_optimism_evm::{NullRefundPolicy, OpEvmFactory, OpTx, PostExecExecutedTx, PostExecTxContext};
use alloy_op_evm::post_exec::PostExecEvmFactoryHooks;

type Stock = OpEvmFactory<OpTx, NullRefundPolicy>;

/// Like [`OpEvmFactory`], but injects Quub F201–F203 into the precompile map.
#[derive(Debug, Clone, Copy, Default)]
pub struct QuubOpEvmFactory;

impl EvmFactory for QuubOpEvmFactory {
    type Evm<DB: Database, I: Inspector<<Stock as EvmFactory>::Context<DB>>> =
        <Stock as EvmFactory>::Evm<DB, I>;
    type Context<DB: Database> = <Stock as EvmFactory>::Context<DB>;
    type Tx = <Stock as EvmFactory>::Tx;
    type Error<DBError: DBErrorMarker> = <Stock as EvmFactory>::Error<DBError>;
    type HaltReason = <Stock as EvmFactory>::HaltReason;
    type Spec = <Stock as EvmFactory>::Spec;
    type BlockEnv = <Stock as EvmFactory>::BlockEnv;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(
        &self,
        db: DB,
        input: EvmEnv<Self::Spec, Self::BlockEnv>,
    ) -> Self::Evm<DB, NoOpInspector> {
        let mut evm = Stock::default().create_evm(db, input);
        let (_, _, map) = evm.components_mut();
        inject_quub_precompiles(map);
        evm
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>>>(
        &self,
        db: DB,
        input: EvmEnv<Self::Spec, Self::BlockEnv>,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        let mut evm = Stock::default().create_evm_with_inspector(db, input, inspector);
        let (_, _, map) = evm.components_mut();
        inject_quub_precompiles(map);
        evm
    }
}

impl PostExecEvmFactoryHooks for QuubOpEvmFactory {
    type Snapshot = ();

    fn begin_post_exec_tx<DB, I>(evm: &mut Self::Evm<DB, I>, ctx: PostExecTxContext)
    where
        DB: Database,
        I: Inspector<Self::Context<DB>>,
    {
        evm.begin_post_exec_tx(ctx);
    }

    fn take_last_post_exec_tx_result<DB, I>(evm: &mut Self::Evm<DB, I>) -> PostExecExecutedTx
    where
        DB: Database,
        I: Inspector<Self::Context<DB>>,
    {
        evm.take_last_post_exec_tx_result()
    }

    fn refund_snapshot<DB, I>(evm: &Self::Evm<DB, I>) -> Self::Snapshot
    where
        DB: Database,
        I: Inspector<Self::Context<DB>>,
    {
        let _ = evm.refund_snapshot();
        ()
    }

    fn seed_refund_snapshot<DB, I>(evm: &mut Self::Evm<DB, I>, _state: Self::Snapshot)
    where
        DB: Database,
        I: Inspector<Self::Context<DB>>,
    {
        evm.seed_refund_snapshot(());
    }
}
