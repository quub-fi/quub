//! Quub Gateway — multi-rail router.
//!
//! Sprint 6: CCTP V2 burns USDC on Ethereum Sepolia and mints on Base Sepolia.
//! Quub 8091 only records identity (`transferWithMemo`). Solana stays a stub.

mod base;
mod cctp;
mod record;

pub use base::{BaseDest, MOCK_BASE_URI};
pub use cctp::{
    Attestation, BurnResult, CctpAdapter, CctpAddresses, CctpError, CctpMode, MintResult,
    BASE_SEPOLIA_DOMAIN, CIRCLE_EVM_CONTRACTS_URL, ETH_SEPOLIA_DOMAIN, MESSAGE_TRANSMITTER_V2,
    QUUB_CHAIN_ID, TOKEN_MESSENGER_V2, USDC_BASE_SEPOLIA, USDC_ETH_SEPOLIA,
};
pub use record::{RailRecord, RailStatus, RailStore, RecordError};

use alloy_primitives::{Address, B256, U256};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentIntent {
    pub rail: &'static str,
    pub token: Address,
    pub from: Address,
    pub to: Address,
    pub amount: U256,
    pub memo_hash: B256,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SendReceipt {
    pub rail: &'static str,
    pub intent_id: u64,
    pub accepted: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RailError {
    #[error("rejected by stub adapter")]
    Rejected,
    #[error("zero amount")]
    ZeroAmount,
}

/// Multi-rail send interface (Sprint 0 stubs).
pub trait Rail {
    fn name(&self) -> &'static str;
    fn send(&mut self, intent: &PaymentIntent) -> Result<SendReceipt, RailError>;
}

/// Base (OP Stack L2) stub — does not dial RPC in unit tests.
#[derive(Default, Debug)]
pub struct BaseAdapter {
    pub sent: Vec<PaymentIntent>,
    next_id: u64,
}

impl BaseAdapter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Rail for BaseAdapter {
    fn name(&self) -> &'static str {
        "base"
    }

    fn send(&mut self, intent: &PaymentIntent) -> Result<SendReceipt, RailError> {
        if intent.amount.is_zero() {
            return Err(RailError::ZeroAmount);
        }
        self.next_id += 1;
        self.sent.push(intent.clone());
        Ok(SendReceipt {
            rail: self.name(),
            intent_id: self.next_id,
            accepted: true,
        })
    }
}

/// Solana stub — does not dial RPC in unit tests. Unchanged for Sprint 6.
#[derive(Default, Debug)]
pub struct SolanaAdapter {
    pub sent: Vec<PaymentIntent>,
    next_id: u64,
}

impl SolanaAdapter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Rail for SolanaAdapter {
    fn name(&self) -> &'static str {
        "solana"
    }

    fn send(&mut self, intent: &PaymentIntent) -> Result<SendReceipt, RailError> {
        if intent.amount.is_zero() {
            return Err(RailError::ZeroAmount);
        }
        self.next_id += 1;
        self.sent.push(intent.clone());
        Ok(SendReceipt {
            rail: self.name(),
            intent_id: self.next_id,
            accepted: true,
        })
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RailFlowError {
    #[error(transparent)]
    Cctp(#[from] CctpError),
    #[error(transparent)]
    Record(#[from] RecordError),
    #[error("policy rejected — CCTP burn not called")]
    PolicyRejected,
}

/// Orchestrate Quub identity + CCTP. Idempotent on `end_to_end_id`.
pub struct RailFlow {
    pub store: RailStore,
    pub cctp: CctpAdapter,
    pub dest: BaseDest,
}

impl RailFlow {
    pub fn mock() -> Self {
        Self {
            store: RailStore::new(),
            cctp: CctpAdapter::mock(),
            dest: BaseDest::mock(),
        }
    }

    /// Record Quub memo result. If `end_to_end_id` already exists, returns the stored record
    /// without requiring a second Quub memo.
    pub fn record_quub(
        &mut self,
        end_to_end_id: B256,
        quub_tx: B256,
        memo_hash: B256,
    ) -> &RailRecord {
        self.store
            .insert_quub(end_to_end_id, quub_tx, memo_hash, self.dest.rpc.clone())
    }

    /// Run CCTP after Quub success. Policy reject skips burn entirely.
    pub fn continue_cctp(
        &mut self,
        end_to_end_id: B256,
        policy_allowed: bool,
        amount_usdc: u128,
        mint_recipient: Address,
    ) -> Result<RailRecord, RailFlowError> {
        let rec = self
            .store
            .get(&end_to_end_id)
            .ok_or(RecordError::Unknown)?
            .clone();

        // Idempotent: already minted → return as-is (no double-burn).
        if rec.status == RailStatus::Minted {
            return Ok(rec);
        }

        // Already burned → retry mint only (no second burn).
        if rec.status == RailStatus::Burned || rec.burn_tx.is_some() {
            let att = Attestation {
                attestation: b"retry-mock-attestation".to_vec(),
            };
            let mint = self.cctp.mint(&[], &att, rec.quub_tx)?;
            self.store.set_minted(&end_to_end_id, mint.mint_tx)?;
            return Ok(self.store.get(&end_to_end_id).unwrap().clone());
        }

        // Policy reject → burn never called (keep Quub memo / status quub_only).
        if !policy_allowed {
            return Err(RailFlowError::PolicyRejected);
        }

        let burn = match self.cctp.burn(
            USDC_ETH_SEPOLIA,
            amount_usdc,
            BASE_SEPOLIA_DOMAIN,
            mint_recipient,
            rec.quub_tx,
        ) {
            Ok(b) => {
                self.store.set_burned(&end_to_end_id, b.burn_tx)?;
                b
            }
            Err(e) => {
                let _ = self.store.set_failed(&end_to_end_id);
                return Err(RailFlowError::Cctp(e));
            }
        };

        let att = match self.cctp.attest(&burn.message_bytes) {
            Ok(a) => a,
            Err(e) => {
                let _ = self.store.set_failed(&end_to_end_id);
                return Err(RailFlowError::Cctp(e));
            }
        };

        match self.cctp.mint(&burn.message_bytes, &att, rec.quub_tx) {
            Ok(m) => {
                self.store.set_minted(&end_to_end_id, m.mint_tx)?;
                Ok(self.store.get(&end_to_end_id).unwrap().clone())
            }
            Err(e) => {
                let _ = self.store.set_failed(&end_to_end_id);
                Err(RailFlowError::Cctp(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, fixed_bytes, uint};

    fn sample_intent(rail: &'static str) -> PaymentIntent {
        PaymentIntent {
            rail,
            token: address!("0x000000000000000000000000000000000000F210"),
            from: address!("0x0000000000000000000000000000000000000001"),
            to: address!("0x0000000000000000000000000000000000000002"),
            amount: uint!(50_U256),
            memo_hash: fixed_bytes!(
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            ),
        }
    }

    #[test]
    fn base_adapter_records_without_network() {
        let mut rail = BaseAdapter::new();
        let intent = sample_intent("base");
        let receipt = rail.send(&intent).expect("send");
        assert_eq!(receipt.rail, "base");
        assert!(receipt.accepted);
        assert_eq!(rail.sent.len(), 1);
    }

    #[test]
    fn solana_adapter_records_without_network() {
        let mut rail = SolanaAdapter::new();
        let intent = sample_intent("solana");
        let receipt = rail.send(&intent).expect("send");
        assert_eq!(receipt.rail, "solana");
        assert!(receipt.accepted);
        assert_eq!(rail.sent.len(), 1);
    }

    #[test]
    fn policy_reject_never_calls_burn() {
        let mut flow = RailFlow::mock();
        let e2e = fixed_bytes!(
            "1111111111111111111111111111111111111111111111111111111111111111"
        );
        let quub = fixed_bytes!(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        let memo = fixed_bytes!(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
        flow.record_quub(e2e, quub, memo);
        let before = flow.cctp.burn_calls;
        let err = flow
            .continue_cctp(e2e, false, 1_000_000, Address::ZERO)
            .unwrap_err();
        assert_eq!(err, RailFlowError::PolicyRejected);
        assert_eq!(flow.cctp.burn_calls, before);
    }

    #[test]
    fn idempotent_end_to_end_no_double_burn() {
        let mut flow = RailFlow::mock();
        let e2e = fixed_bytes!(
            "2222222222222222222222222222222222222222222222222222222222222222"
        );
        let quub = fixed_bytes!(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        let memo = fixed_bytes!(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
        flow.record_quub(e2e, quub, memo);
        let r1 = flow
            .continue_cctp(e2e, true, 1_000_000, Address::ZERO)
            .unwrap();
        assert_eq!(r1.status, RailStatus::Minted);
        let burns = flow.cctp.burn_calls;
        let r2 = flow
            .continue_cctp(e2e, true, 1_000_000, Address::ZERO)
            .unwrap();
        assert_eq!(r2.status, RailStatus::Minted);
        assert_eq!(r2.burn_tx, r1.burn_tx);
        assert_eq!(flow.cctp.burn_calls, burns);
        assert_ne!(r1.burn_tx.unwrap(), quub);
    }

    #[test]
    fn mock_mint_records_dest_hash() {
        let mut flow = RailFlow::mock();
        let e2e = fixed_bytes!(
            "3333333333333333333333333333333333333333333333333333333333333333"
        );
        let quub = fixed_bytes!(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        let memo = fixed_bytes!(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
        flow.record_quub(e2e, quub, memo);
        let r = flow
            .continue_cctp(e2e, true, 500_000, Address::ZERO)
            .unwrap();
        assert_eq!(r.status, RailStatus::Minted);
        assert!(r.mint_tx.is_some());
        assert_ne!(r.mint_tx.unwrap(), quub);
        assert_eq!(r.dest, MOCK_BASE_URI);
    }

    #[test]
    fn second_record_quub_reuses_without_new_tx() {
        let mut flow = RailFlow::mock();
        let e2e = fixed_bytes!(
            "4444444444444444444444444444444444444444444444444444444444444444"
        );
        let quub = fixed_bytes!(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        let memo = fixed_bytes!(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
        let a = flow.record_quub(e2e, quub, memo).quub_tx;
        let other = fixed_bytes!(
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        );
        let b = flow.record_quub(e2e, other, memo).quub_tx;
        assert_eq!(a, b);
        assert_eq!(a, quub);
    }
}
