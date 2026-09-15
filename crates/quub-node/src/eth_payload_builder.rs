//! PayloadServiceBuilder that installs [`QuubEthPayloadBuilder`] on `--dev`.

use crate::eth_payload::QuubEthPayloadBuilder;
use reth_ethereum::{
    chainspec::{EthChainSpec, EthereumHardforks},
    evm::primitives::{ConfigureEvm, NextBlockEnvAttributes},
    node::{
        api::{FullNodeTypes, NodeTypes, PrimitivesTy, TxTy},
        builder::{
            components::PayloadBuilderBuilder, BuilderContext, PayloadBuilderConfig, PayloadTypes,
        },
    },
    pool::{PoolTransaction, TransactionPool},
    EthPrimitives,
};
use reth_ethereum_engine_primitives::{EthBuiltPayload, EthPayloadAttributes};
use reth_ethereum_payload_builder::EthereumBuilderConfig;

/// Unit builder plugged into `EthereumNode::components().payload(...)`.
#[derive(Clone, Default, Debug)]
#[non_exhaustive]
pub struct QuubEthPayloadServiceBuilder;

impl<Types, Node, Pool, Evm> PayloadBuilderBuilder<Node, Pool, Evm> for QuubEthPayloadServiceBuilder
where
    Types: NodeTypes<ChainSpec: EthereumHardforks, Primitives = EthPrimitives>,
    Node: FullNodeTypes<Types = Types>,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TxTy<Node::Types>>>
        + Unpin
        + 'static,
    Evm: ConfigureEvm<Primitives = PrimitivesTy<Types>, NextBlockEnvCtx = NextBlockEnvAttributes>
        + Clone
        + 'static,
    Types::Payload:
        PayloadTypes<BuiltPayload = EthBuiltPayload, PayloadAttributes = EthPayloadAttributes>,
{
    type PayloadBuilder = QuubEthPayloadBuilder<Pool, Node::Provider, Evm>;

    async fn build_payload_builder(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
        evm_config: Evm,
    ) -> eyre::Result<Self::PayloadBuilder> {
        let conf = ctx.payload_builder_config();
        let chain = ctx.chain_spec().chain();
        let gas_limit = conf.gas_limit_for(chain);
        let skip_state_root = ctx.config().tree_config().skip_state_root();

        Ok(QuubEthPayloadBuilder::new(
            ctx.provider().clone(),
            pool,
            evm_config,
            EthereumBuilderConfig::new()
                .with_gas_limit(gas_limit)
                .with_max_blobs_per_block(conf.max_blobs_per_block())
                .with_extra_data(conf.extra_data())
                .with_skip_state_root(skip_state_root),
        ))
    }
}
