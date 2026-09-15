//! Mode A (OP Stack / Engine API).
//!
//! Sprint 2: **blocked** on the locked Reth pin. See [`CONFLICT.md`](../CONFLICT.md).
//! Do not start op-node from this crate until the conflict is resolved.

/// Compile-time placeholder so `mode-a` feature links.
pub fn placeholder() {}

/// Human-readable block reason for CLI / scripts.
pub const MODE_A_BLOCKED: &str = concat!(
    "Mode A blocked: paradigmxyz/reth v2.5.2 has no reth-optimism-* crates; ",
    "op-reth (ethereum-optimism/optimism, latest checked op-reth/v2.4.4) pins op-rs/reth, ",
    "not paradigmxyz/reth@v2.5.2. Do not bump Reth. See crates/quub-consensus-op/CONFLICT.md"
);

/// Returns the conflict message. Never starts Engine API.
pub fn engine_unavailable_reason() -> &'static str {
    MODE_A_BLOCKED
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflict_message_mentions_reth_pin() {
        assert!(MODE_A_BLOCKED.contains("v2.5.2"));
        assert!(MODE_A_BLOCKED.contains("op-reth"));
    }
}
