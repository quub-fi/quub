//! In-memory evidence store. On-chain counterpart is EvidenceAnchor (F212).
//!
//! Stores `(packHash, memoHash)` only — never ISO XML or PII.

use alloy_primitives::B256;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EvidenceError {
    #[error("zero packHash")]
    ZeroPackHash,
    #[error("zero memoHash")]
    ZeroMemoHash,
    #[error("already anchored")]
    AlreadyAnchored,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorRecord {
    pub pack_hash: B256,
    pub memo_hash: B256,
}

/// In-memory evidence plane for Sprint 0.
#[derive(Default, Debug)]
pub struct EvidenceStore {
    by_pack: HashMap<B256, AnchorRecord>,
}

impl EvidenceStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// `anchor(bytes32 packHash, bytes32 memoHash)`.
    pub fn anchor(&mut self, pack_hash: B256, memo_hash: B256) -> Result<(), EvidenceError> {
        if pack_hash == B256::ZERO {
            return Err(EvidenceError::ZeroPackHash);
        }
        if memo_hash == B256::ZERO {
            return Err(EvidenceError::ZeroMemoHash);
        }
        if self.by_pack.contains_key(&pack_hash) {
            return Err(EvidenceError::AlreadyAnchored);
        }
        self.by_pack.insert(
            pack_hash,
            AnchorRecord {
                pack_hash,
                memo_hash,
            },
        );
        Ok(())
    }

    pub fn get(&self, pack_hash: &B256) -> Option<&AnchorRecord> {
        self.by_pack.get(pack_hash)
    }

    pub fn len(&self) -> usize {
        self.by_pack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_pack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::fixed_bytes;

    #[test]
    fn anchor_stores_pack_and_memo() {
        let mut store = EvidenceStore::new();
        let pack = fixed_bytes!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let memo = fixed_bytes!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        store.anchor(pack, memo).expect("anchor");
        let rec = store.get(&pack).expect("present");
        assert_eq!(rec.pack_hash, pack);
        assert_eq!(rec.memo_hash, memo);
        assert_eq!(store.len(), 1);
    }
}
