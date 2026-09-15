//! Mode A (OP Stack / Engine API) — ADR-016.
//!
//! History of the prior pin conflict: see [`CONFLICT.md`](../CONFLICT.md).

mod launch;
mod op_factory;
mod quub_op_executor;
mod quub_op_node;

pub use launch::{EngineArgs, launch_engine};
pub use op_factory::QuubOpEvmFactory;
pub use quub_op_executor::QuubOpExecutorBuilder;
pub use quub_op_node::QuubOpNode;

#[cfg(test)]
mod tests {
    #[test]
    fn conflict_doc_mentions_adr016() {
        let doc = include_str!("../CONFLICT.md");
        assert!(doc.contains("ADR-016"));
        assert!(doc.contains("aef8d3ef"));
    }
}
