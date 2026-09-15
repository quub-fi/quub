//! quub-node — `--dev` HTTP on 8545 (chain 8091); `--engine` blocked until Mode A pin resolves.

mod alloc_genesis;
mod launch;

fn main() -> eyre::Result<()> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--engine" => {
                #[cfg(feature = "mode-a")]
                {
                    eprintln!("{}", quub_consensus_op::engine_unavailable_reason());
                    std::process::exit(1);
                }
                #[cfg(not(feature = "mode-a"))]
                {
                    eprintln!("quub-node: built without mode-a; --engine unavailable");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                eprintln!(
                    "quub-node\n  (default) --dev-style HTTP on 127.0.0.1:8545, chain 8091\n  --engine  Mode A (blocked; see crates/quub-consensus-op/CONFLICT.md)"
                );
                return Ok(());
            }
            other => {
                eprintln!("quub-node: unknown argument {other}");
                std::process::exit(2);
            }
        }
    }

    // Default: Sprint 1.5 --dev path.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(launch::launch())
}
