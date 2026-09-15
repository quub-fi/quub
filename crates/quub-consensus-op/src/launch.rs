//! Mode A Engine API launch (`quub-node --engine`).

use crate::quub_op_node::QuubOpNode;
use alloy_genesis::Genesis;
use reth_ethereum::{
    node::{
        builder::{NodeBuilder, NodeHandle},
        core::{args::RpcServerArgs, node_config::NodeConfig},
    },
    tasks::Runtime,
};
use reth_optimism_chainspec::OpChainSpec;
use reth_optimism_node::args::RollupArgs;
use std::{net::Ipv4Addr, path::PathBuf, sync::Arc};

/// CLI args for `--engine`.
#[derive(Debug, Clone)]
pub struct EngineArgs {
    pub http_port: u16,
    pub auth_port: u16,
    pub jwt_path: PathBuf,
    pub genesis_path: PathBuf,
}

impl Default for EngineArgs {
    fn default() -> Self {
        Self {
            http_port: 9545,
            auth_port: 9551,
            jwt_path: PathBuf::from("artifacts/mode-a/jwt.txt"),
            genesis_path: PathBuf::from("artifacts/mode-a/deployer/l2-genesis-quub.json"),
        }
    }
}

/// Launch OpNode + Quub precompiles. No `--dev` miner — blocks via Engine API only.
pub async fn launch_engine(args: EngineArgs) -> eyre::Result<()> {
    let genesis_json = std::fs::read_to_string(&args.genesis_path).map_err(|e| {
        eyre::eyre!("read L2 genesis {}: {e}", args.genesis_path.display())
    })?;
    let genesis: Genesis =
        serde_json::from_str(&genesis_json).map_err(|e| eyre::eyre!("parse L2 genesis: {e}"))?;
    let chain_spec = Arc::new(OpChainSpec::from(genesis));

    let mut rpc = RpcServerArgs::default().with_http();
    rpc.http_addr = Ipv4Addr::LOCALHOST.into();
    rpc.http_port = args.http_port;
    rpc.auth_addr = Ipv4Addr::LOCALHOST.into();
    rpc.auth_port = args.auth_port;
    rpc.auth_jwtsecret = Some(args.jwt_path.clone());
    rpc.ws_port = 0;

    let mut node_config = NodeConfig::new(chain_spec).with_rpc(rpc);
    node_config.rpc.http = true;
    node_config.rpc.http_addr = Ipv4Addr::LOCALHOST.into();
    node_config.rpc.http_port = args.http_port;
    node_config.rpc.auth_port = args.auth_port;
    node_config.rpc.auth_addr = Ipv4Addr::LOCALHOST.into();
    node_config.rpc.auth_jwtsecret = Some(args.jwt_path.clone());
    node_config.rpc.ws_port = 0;

    eprintln!(
        "quub-node: http={}:{} authrpc={}:{} jwt={} chain=8091 mode=--engine",
        Ipv4Addr::LOCALHOST,
        args.http_port,
        Ipv4Addr::LOCALHOST,
        args.auth_port,
        args.jwt_path.display()
    );
    eprintln!(
        "quub-node: genesis={} datadir={}",
        args.genesis_path.display(),
        node_config.datadir().data_dir().display()
    );

    let runtime = Runtime::test();
    let rollup_args = RollupArgs::default();

    let NodeHandle {
        node,
        node_exit_future,
    } = NodeBuilder::new(node_config)
        .testing_node(runtime)
        .node(QuubOpNode::new(rollup_args))
        .launch()
        .await?;

    if let Some(addr) = node.rpc_server_handle().http_local_addr() {
        eprintln!("quub-node: rpc http listening on {addr}");
    } else {
        eprintln!("quub-node: WARNING — HTTP RPC did not start");
    }

    node_exit_future.await?;
    Ok(())
}
