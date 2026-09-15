//! quub-node — `--dev` HTTP on 8545 (chain 8091); `--engine` Mode A on 9545/9551.

mod alloc_genesis;
mod eth_payload;
mod eth_payload_builder;
mod launch;

fn main() -> eyre::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut engine = false;
    let mut http_port = 9545u16;
    let mut auth_port = 9551u16;
    let mut jwt_path = std::path::PathBuf::from("artifacts/mode-a/jwt.txt");
    let mut genesis_path =
        std::path::PathBuf::from("artifacts/mode-a/deployer/l2-genesis-quub.json");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--engine" => engine = true,
            "--http-port" => {
                http_port = args
                    .next()
                    .ok_or_else(|| eyre::eyre!("--http-port needs a value"))?
                    .parse()?;
            }
            "--auth-port" | "--authrpc-port" => {
                auth_port = args
                    .next()
                    .ok_or_else(|| eyre::eyre!("--auth-port needs a value"))?
                    .parse()?;
            }
            "--jwt" => {
                jwt_path = args
                    .next()
                    .ok_or_else(|| eyre::eyre!("--jwt needs a path"))?
                    .into();
            }
            "--genesis" => {
                genesis_path = args
                    .next()
                    .ok_or_else(|| eyre::eyre!("--genesis needs a path"))?
                    .into();
            }
            "--help" | "-h" => {
                eprintln!(
                    "quub-node\n  (default) --dev-style HTTP on 127.0.0.1:8545, chain 8091\n  --engine [--http-port 9545] [--auth-port 9551] [--jwt PATH] [--genesis PATH]"
                );
                return Ok(());
            }
            other => {
                eprintln!("quub-node: unknown argument {other}");
                std::process::exit(2);
            }
        }
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    if engine {
        #[cfg(feature = "mode-a")]
        {
            return rt.block_on(quub_consensus_op::launch_engine(
                quub_consensus_op::EngineArgs {
                    http_port,
                    auth_port,
                    jwt_path,
                    genesis_path,
                },
            ));
        }
        #[cfg(not(feature = "mode-a"))]
        {
            eprintln!("quub-node: built without mode-a; --engine unavailable");
            std::process::exit(1);
        }
    }

    rt.block_on(launch::launch())
}
