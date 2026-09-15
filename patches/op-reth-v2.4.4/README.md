# Local patches on optimism `op-reth/v2.4.4`

Applied under `/tmp/optimism-op-node` (path deps in workspace `Cargo.toml`):

1. `reth-optimism-evm`: `ConfigureEngineEvm<OpExecutionData>` for `OpEvmConfig<…, PostExecEvmFactoryAdapter<F>>`
2. `reth-optimism-payload-builder`: `ConfigureEngineEvm<OpExecData>` bridge for the same adapter

Needed so Quub can inject F201–F203 via `PostExecEvmFactoryAdapter<QuubOpEvmFactory>` (UniEvmFactory pattern) while Engine API still compiles.

Copies of the patched `lib.rs` files are in this directory for reference.
