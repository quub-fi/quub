//! Quub Gateway — multi-rail router stubs.
//!
//! Unit tests must not hit the network. Adapters record intent only.
//!
//! Locked: Quub Gateway runs against public Base/Solana stubs with no Quub
//! node required for off-chain rails (AGENTS.md §0 / Sprint 0).

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

/// Multi-rail send interface.
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

/// Solana stub — does not dial RPC in unit tests.
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
}
