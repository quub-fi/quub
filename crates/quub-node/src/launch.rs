//! Programmatic EthereumNode launch. Does not call `quub-consensus-op`.
//!
//! Pattern: reth `examples/custom-evm` + `examples/custom-dev-node`
//! (`launch_with_debug_capabilities` for `--dev` mining).

use crate::alloc_genesis::quub_genesis;
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
use std::net::Ipv4Addr;
use std::sync::Arc;

pub fn chain_spec() -> Arc<ChainSpec> {
    let spec = ChainSpec::builder()
        .chain(Chain::from_id(8091))
        .genesis(quub_genesis())
        .london_activated()
        .paris_activated()
        .shanghai_activated()
        .cancun_activated()
        .prague_activated()
        .build();
    Arc::new(spec)
}

pub async fn launch() -> eyre::Result<()> {
    let runtime = Runtime::test();

    let mut rpc = RpcServerArgs::default().with_http();
    rpc.http_addr = Ipv4Addr::LOCALHOST.into();
    rpc.http_port = 8545;
    rpc.auth_port = 0;
    rpc.ws_port = 0;

    let mut node_config = NodeConfig::test()
        .with_chain(chain_spec())
        .dev()
        .with_rpc(rpc);
    node_config.rpc.http = true;
    node_config.rpc.http_addr = Ipv4Addr::LOCALHOST.into();
    node_config.rpc.http_port = 8545;
    node_config.rpc.auth_port = 0;
    node_config.rpc.ws_port = 0;

    eprintln!(
        "quub-node: http={}:{} chain=8091 mode=--dev",
        node_config.rpc.http_addr, node_config.rpc.http_port
    );
    // `NodeConfig::test()` uses an ephemeral datadir; receipts do not survive restart.
    eprintln!(
        "quub-node: datadir={} (ephemeral --dev; wiped on exit)",
        node_config.datadir().data_dir().display()
    );

    let NodeHandle {
        node,
        node_exit_future,
    } = NodeBuilder::new(node_config)
        .testing_node(runtime)
        .with_types::<EthereumNode>()
        .with_components(EthereumNode::components().executor(QuubExecutorBuilder::default()))
        .with_add_ons(EthereumAddOns::default())
        .launch_with_debug_capabilities()
        .await?;

    if let Some(addr) = node.rpc_server_handle().http_local_addr() {
        eprintln!("quub-node: rpc http listening on {addr}");
    } else {
        eprintln!("quub-node: WARNING — HTTP RPC did not start");
    }

    node_exit_future.await?;
    Ok(())
}
