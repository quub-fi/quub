//! quub-node — Sprint 1 programmatic `--dev` HTTP on 8545, chain id 8091.

mod launch;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    launch::launch().await
}
