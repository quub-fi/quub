//! Quub OP node: Quub executor + lane-ordered payload txs (ADR-017).

use crate::lane_txs::QuubLaneTxs;
use crate::quub_op_executor::QuubOpExecutorBuilder;
use reth_ethereum::node::{
    api::FullNodeTypes,
    builder::{
        components::{BasicPayloadServiceBuilder, ComponentsBuilder, NodeComponentsBuilder},
        rpc::BasicEngineValidatorBuilder,
        Node, NodeAdapter, NodeTypes as RethNodeTypes,
    },
};
use reth_optimism_node::{
    args::RollupArgs,
    node::{
        OpAddOns, OpConsensusBuilder, OpEngineValidatorBuilder, OpFullNodeTypes, OpNetworkBuilder,
        OpNode, OpNodeTypes, OpPayloadBuilder, OpPoolBuilder,
    },
    rpc::OpEthApiBuilder,
    OpEngineApiBuilder,
};

/// [`OpNode`] with Quub precompiles and payment-lane tx ordering.
#[derive(Debug, Clone)]
pub struct QuubOpNode {
    inner: OpNode,
}

impl QuubOpNode {
    pub fn new(args: RollupArgs) -> Self {
        Self {
            inner: OpNode::new(args),
        }
    }
}

impl RethNodeTypes for QuubOpNode {
    type Primitives = <OpNode as RethNodeTypes>::Primitives;
    type ChainSpec = <OpNode as RethNodeTypes>::ChainSpec;
    type Storage = <OpNode as RethNodeTypes>::Storage;
    type Payload = <OpNode as RethNodeTypes>::Payload;
}

impl<N> Node<N> for QuubOpNode
where
    N: FullNodeTypes<Types: OpFullNodeTypes + OpNodeTypes>,
{
    type ComponentsBuilder = ComponentsBuilder<
        N,
        OpPoolBuilder,
        BasicPayloadServiceBuilder<OpPayloadBuilder<QuubLaneTxs>>,
        OpNetworkBuilder,
        QuubOpExecutorBuilder,
        OpConsensusBuilder,
    >;

    type AddOns = OpAddOns<
        NodeAdapter<N, <Self::ComponentsBuilder as NodeComponentsBuilder<N>>::Components>,
        OpEthApiBuilder,
        OpEngineValidatorBuilder,
        OpEngineApiBuilder<OpEngineValidatorBuilder>,
        BasicEngineValidatorBuilder<OpEngineValidatorBuilder>,
    >;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        OpNode::components::<N>(&self.inner)
            .executor(QuubOpExecutorBuilder)
            .payload(BasicPayloadServiceBuilder::new(
                self.inner.payload_builder().with_transactions(QuubLaneTxs::new()),
            ))
    }

    fn add_ons(&self) -> Self::AddOns {
        self.inner.add_ons_builder().build()
    }
}
