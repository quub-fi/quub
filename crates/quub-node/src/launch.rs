//! Programmatic EthereumNode launch. Does not call `quub-consensus-op`.

use alloy_genesis::Genesis;
use quub_evm::QuubExecutorBuilder;
use reth_ethereum::{
    chainspec::{Chain, ChainSpec},
    node::{
        builder::{NodeBuilder, NodeHandle},
        core::{args::RpcServerArgs, node_config::NodeConfig},
        node::EthereumAddOns,
        EthereumNode,
    },
    tasks::Runtime,
};
use std::sync::Arc;

pub fn chain_spec() -> Arc<ChainSpec> {
    let spec = ChainSpec::builder()
        .chain(Chain::from_id(8091))
        .genesis(Genesis::default())
        .london_activated()
        .paris_activated()
        .shanghai_activated()
        .cancun_activated()
        .prague_activated()
        .build();
    Arc::new(spec)
}

pub async fn launch() -> eyre::Result<()> {
    let runtime = Runtime::try_current().unwrap_or_else(|_| Runtime::test());

    let node_config = NodeConfig::test()
        .dev()
        .with_rpc(
            RpcServerArgs::default()
                .with_http()
                .with_http_addr("127.0.0.1".parse().expect("http addr"))
                .with_http_port(8545),
        )
        .with_chain(chain_spec());

    let NodeHandle { node: _, node_exit_future } = NodeBuilder::new(node_config)
        .testing_node(runtime)
        .with_types::<EthereumNode>()
        .with_components(EthereumNode::components().executor(QuubExecutorBuilder::default()))
        .with_add_ons(EthereumAddOns::default())
        .launch()
        .await?;

    node_exit_future.await?;
    Ok(())
}
