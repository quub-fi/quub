//! Programmatic EthereumNode launch. Does not call `quub-consensus-op`.
//!
//! Pattern: reth `examples/custom-evm` + `examples/custom-dev-node`
//! (`launch_with_debug_capabilities` for `--dev` mining).

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
use std::net::Ipv4Addr;
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
    let runtime = Runtime::test();

    // HTTP fixed at 8545; auth/ws ports OS-assigned so leftover processes do not block.
    let mut rpc = RpcServerArgs::default().with_http();
    rpc.http_addr = Ipv4Addr::LOCALHOST.into();
    rpc.http_port = 8545;
    rpc.auth_port = 0;
    rpc.ws_port = 0;

    let mut node_config = NodeConfig::test()
        .with_chain(chain_spec())
        .dev()
        .with_rpc(rpc);
    // Re-assert after NodeConfig::test()'s with_unused_ports / with_rpc.
    node_config.rpc.http = true;
    node_config.rpc.http_addr = Ipv4Addr::LOCALHOST.into();
    node_config.rpc.http_port = 8545;
    node_config.rpc.auth_port = 0;
    node_config.rpc.ws_port = 0;

    eprintln!(
        "quub-node: http={}:{}",
        node_config.rpc.http_addr, node_config.rpc.http_port
    );

    // Keep `node` alive: RpcServerHandle stops HTTP when dropped.
    let NodeHandle { node, node_exit_future } = NodeBuilder::new(node_config)
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
