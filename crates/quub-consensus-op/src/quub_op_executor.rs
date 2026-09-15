//! Quub OP executor builder (UniEvmFactory pattern from op-reth tests).

use crate::op_factory::QuubOpEvmFactory;
use reth_ethereum::node::{
    api::{FullNodeTypes, NodeTypes},
    builder::{components::ExecutorBuilder, BuilderContext},
};
use reth_optimism_evm::{
    OpBlockExecutorFactory, OpEvmConfig, OpRethReceiptBuilder, PostExecEvmFactoryAdapter,
};
use reth_optimism_forks::OpHardforks;
use reth_optimism_node::OpExecutorBuilder;
use reth_optimism_primitives::OpPrimitives;
use std::marker::PhantomData;

/// Builds [`OpEvmConfig`] with Quub F201–F203 via [`PostExecEvmFactoryAdapter`].
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct QuubOpExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for QuubOpExecutorBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec: OpHardforks, Primitives = OpPrimitives>>,
{
    type EVM = OpEvmConfig<
        <Node::Types as NodeTypes>::ChainSpec,
        <Node::Types as NodeTypes>::Primitives,
        OpRethReceiptBuilder,
        PostExecEvmFactoryAdapter<QuubOpEvmFactory>,
    >;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        let OpEvmConfig {
            executor_factory,
            block_assembler,
            _pd: _,
        } = OpExecutorBuilder::default().build_evm(ctx).await?;
        let quub_executor_factory = OpBlockExecutorFactory::new(
            *executor_factory.receipt_builder(),
            ctx.chain_spec(),
            PostExecEvmFactoryAdapter::new(QuubOpEvmFactory),
        );
        Ok(OpEvmConfig {
            executor_factory: quub_executor_factory,
            block_assembler,
            _pd: PhantomData,
        })
    }
}
