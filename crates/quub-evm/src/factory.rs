//! QuubEvmFactory — copy of reth v2.5.2 examples/custom-evm, with spec >= PRAGUE.

use crate::precompiles::precompiles_for_spec;
use alloy_evm::{eth::EthEvmContext, precompiles::PrecompilesMap, EvmFactory};
use reth_ethereum::{
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
    type Error = EVMError<DB::Error>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Spec = SpecId;
    type BlockEnv = BlockEnv;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(
        &self,
        db: DB,
        input: EvmEnv<SpecId>,
    ) -> Self::Evm<DB, NoOpInspector> {
        let spec = input.cfg_env.spec;
        let precompiles = if spec >= SpecId::PRAGUE {
            PrecompilesMap::from_static(precompiles_for_spec(spec))
        } else {
            PrecompilesMap::from_static(
                reth_ethereum::evm::revm::handler::EthPrecompiles::new(spec).precompiles,
            )
        };

        let evm = Context::mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(precompiles);

        EthEvm::new(evm, false)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>>(
        &self,
        db: DB,
        input: EvmEnv<SpecId>,
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
    Node: FullNodeTypes<
        Types: NodeTypes<
            ChainSpec = reth_ethereum::chainspec::ChainSpec,
            Primitives = EthPrimitives,
        >,
    >,
{
    type EVM = EthEvmConfig<reth_ethereum::chainspec::ChainSpec, QuubEvmFactory>;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        Ok(EthEvmConfig::new_with_evm_factory(
            ctx.chain_spec(),
            QuubEvmFactory,
        ))
    }
}
