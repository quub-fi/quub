//! Rail records keyed by `endToEndId` — Quub identity + CCTP burn/mint.

use alloy_primitives::B256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RailStatus {
    QuubOnly,
    Burned,
    Minted,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RailRecord {
    pub end_to_end_id: B256,
    pub quub_tx: B256,
    pub memo_hash: B256,
    pub burn_tx: Option<B256>,
    pub mint_tx: Option<B256>,
    pub status: RailStatus,
    pub dest: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RecordError {
    #[error("burn tx must not equal Quub tx")]
    BurnEqualsQuub,
    #[error("unknown endToEndId")]
    Unknown,
}

#[derive(Default, Debug)]
pub struct RailStore {
    by_e2e: HashMap<B256, RailRecord>,
}

impl RailStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, end_to_end_id: &B256) -> Option<&RailRecord> {
        self.by_e2e.get(end_to_end_id)
    }

    pub fn insert_quub(
        &mut self,
        end_to_end_id: B256,
        quub_tx: B256,
        memo_hash: B256,
        dest: impl Into<String>,
    ) -> &RailRecord {
        self.by_e2e
            .entry(end_to_end_id)
            .or_insert_with(|| RailRecord {
                end_to_end_id,
                quub_tx,
                memo_hash,
                burn_tx: None,
                mint_tx: None,
                status: RailStatus::QuubOnly,
                dest: dest.into(),
            })
    }

    pub fn set_burned(&mut self, end_to_end_id: &B256, burn_tx: B256) -> Result<&RailRecord, RecordError> {
        let rec = self.by_e2e.get_mut(end_to_end_id).ok_or(RecordError::Unknown)?;
        if burn_tx == rec.quub_tx {
            return Err(RecordError::BurnEqualsQuub);
        }
        rec.burn_tx = Some(burn_tx);
        rec.status = RailStatus::Burned;
        Ok(rec)
    }

    pub fn set_minted(&mut self, end_to_end_id: &B256, mint_tx: B256) -> Result<&RailRecord, RecordError> {
        let rec = self.by_e2e.get_mut(end_to_end_id).ok_or(RecordError::Unknown)?;
        rec.mint_tx = Some(mint_tx);
        rec.status = RailStatus::Minted;
        Ok(rec)
    }

    pub fn set_failed(&mut self, end_to_end_id: &B256) -> Result<&RailRecord, RecordError> {
        let rec = self.by_e2e.get_mut(end_to_end_id).ok_or(RecordError::Unknown)?;
        rec.status = RailStatus::Failed;
        Ok(rec)
    }
}
