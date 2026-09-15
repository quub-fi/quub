//! Local precompile error. Not a reth-revm type.

use alloy_primitives::Bytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrecompileError {
    /// Empty revert (unknown selector / rule miss).
    Revert(Bytes),
    OutOfGas,
}

impl PrecompileError {
    pub fn empty_revert() -> Self {
        Self::Revert(Bytes::new())
    }
}
