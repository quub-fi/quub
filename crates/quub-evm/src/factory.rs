//! QuubEvmFactory — copy of reth v2.5.2 examples/custom-evm, with `spec >= PRAGUE`.

use crate::precompiles::precompiles_map_for_spec;
use alloy_evm::{
    eth::EthEvmContext,
    precompiles::PrecompilesMap,
    revm::context::DBErrorMarker,
    EvmFactory,
};
use reth_ethereum::{
    chainspec::ChainSpec,
    evm::{
        primitives::{Database, EvmEnv},
        revm::{
            context::{BlockEnv, Context, TxEnv},
            context_interface::result::{EVMError, HaltReason},
            inspector::{Inspector, NoOpInspector},
            interpreter::interpreter::EthInterpreter,
            primitives::hardfork::SpecId,
            MainBuilder, MainContext,
        },
        EthEvm, EthEvmConfig,
    },
    node::{
        api::{FullNodeTypes, NodeTypes},
        builder::{components::ExecutorBuilder, BuilderContext},
    },
    EthPrimitives,
};

/// Custom EVM factory: Ethereum builtins + F201–F203 when spec >= Prague.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct QuubEvmFactory;

impl EvmFactory for QuubEvmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> =
        EthEvm<DB, I, Self::Precompiles>;
    type Tx = TxEnv;
    type Error<DBError: DBErrorMarker> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Spec = SpecId;
    type BlockEnv = BlockEnv;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(
        &self,
        db: DB,
        input: EvmEnv,
    ) -> Self::Evm<DB, NoOpInspector> {
        let spec = input.cfg_env.spec;
        // Always build via helper so Osaka / later keep F201–F203 (`spec >= PRAGUE`).
        let precompiles = precompiles_map_for_spec(spec);

        let evm = Context::mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(precompiles);

        EthEvm::new(evm, false)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, EthInterpreter>>(
        &self,
        db: DB,
        input: EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        EthEvm::new(
            self.create_evm(db, input).into_inner().with_inspector(inspector),
            true,
        )
    }
}

/// Builds a regular ethereum block executor that uses [`QuubEvmFactory`].
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct QuubExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for QuubExecutorBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>>,
{
    type EVM = EthEvmConfig<ChainSpec, QuubEvmFactory>;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        Ok(EthEvmConfig::new_with_evm_factory(
            ctx.chain_spec(),
            QuubEvmFactory,
        ))
    }
}
