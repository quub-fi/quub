//! eth_* client: chain id, transferWithMemo, freeze/unfreeze, receipts.

use alloy::consensus::{SignableTransaction, TxLegacy};
use alloy::network::TxSignerSync;
use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::signers::local::PrivateKeySigner;
use alloy::transports::http::{Client, Http};
use alloy_primitives::{Address, Bytes, B256, U256};
use alloy_sol_types::{sol, SolCall, SolError, SolEvent};
use quub_primitives::{
    CHAIN_ID_TESTNET, PAYMENT_TOKEN, POLICY_ADMIN,
};

use crate::config::Config;
use crate::reason::reason_name;

sol! {
    #[derive(Debug)]
    function transferWithMemo(
        address to,
        uint256 amount,
        bytes32 endToEndId,
        bytes16 uetr,
        bytes32 instrId,
        bytes3 ccy,
        uint8 msgType,
        bytes32 trHash,
        bytes32 packHash
    ) returns (bytes32 memoHash);

    #[derive(Debug)]
    function freeze(address account);

    #[derive(Debug)]
    function unfreeze(address account);

    #[derive(Debug)]
    error PolicyRejected(uint16 reason);

    #[derive(Debug)]
    event MemoAnchored(bytes32 indexed memoHash, uint8 msgType);
}

type HttpProvider = RootProvider<Http<Client>>;

#[derive(Clone)]
pub struct EthClient {
    provider: HttpProvider,
    signer: PrivateKeySigner,
    chain_id: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum EthError {
    #[error("rpc: {0}")]
    Rpc(String),
    #[error("policy rejected reason={reason}")]
    PolicyRejected { reason: u16 },
    #[error("tx failed: {0}")]
    TxFailed(String),
}

impl EthClient {
    pub async fn connect(config: &Config) -> Result<Self, String> {
        let url = config
            .rpc_url
            .parse()
            .map_err(|e| format!("bad QUUB_RPC url: {e}"))?;
        let provider = ProviderBuilder::new().on_http(url);
        Ok(Self {
            provider,
            signer: config.signer.clone(),
            chain_id: config.chain_id,
        })
    }

    pub fn operator_address(&self) -> Address {
        self.signer.address()
    }

    pub async fn chain_id(&self) -> Result<u64, EthError> {
        self.provider
            .get_chain_id()
            .await
            .map_err(|e| EthError::Rpc(e.to_string()))
    }

    /// Fail-closed health probe: ok only when RPC up and chain is 8091.
    pub async fn health_chain_id(&self) -> Result<u64, EthError> {
        let id = self.chain_id().await?;
        if id != CHAIN_ID_TESTNET {
            return Err(EthError::Rpc(format!(
                "unexpected chainId {id}, expected {CHAIN_ID_TESTNET}"
            )));
        }
        Ok(id)
    }

    pub async fn send_transfer_with_memo(
        &self,
        to: Address,
        amount: U256,
        end_to_end_id: B256,
        uetr: alloy_primitives::FixedBytes<16>,
        instr_id: B256,
        ccy: alloy_primitives::FixedBytes<3>,
        msg_type: u8,
        tr_hash: B256,
        pack_hash: B256,
    ) -> Result<B256, EthError> {
        let call = transferWithMemoCall {
            to,
            amount,
            endToEndId: end_to_end_id,
            uetr,
            instrId: instr_id,
            ccy,
            msgType: msg_type,
            trHash: tr_hash,
            packHash: pack_hash,
        };
        let data = Bytes::from(call.abi_encode());

        // Simulate first to surface PolicyRejected without broadcasting.
        self.simulate_call(PAYMENT_TOKEN, data.clone()).await?;
        self.send_legacy(PAYMENT_TOKEN, data).await
    }

    pub async fn send_freeze(&self, account: Address) -> Result<B256, EthError> {
        let data = Bytes::from(freezeCall { account }.abi_encode());
        self.send_legacy(POLICY_ADMIN, data).await
    }

    pub async fn send_unfreeze(&self, account: Address) -> Result<B256, EthError> {
        let data = Bytes::from(unfreezeCall { account }.abi_encode());
        self.send_legacy(POLICY_ADMIN, data).await
    }

    async fn simulate_call(&self, to: Address, data: Bytes) -> Result<(), EthError> {
        use alloy::rpc::types::TransactionRequest;
        let from = self.signer.address();
        let req = TransactionRequest::default()
            .from(from)
            .to(to)
            .input(data.into());
        match self.provider.call(&req).await {
            Ok(_) => Ok(()),
            Err(e) => Err(classify_rpc_error(&e.to_string())),
        }
    }

    async fn send_legacy(&self, to: Address, data: Bytes) -> Result<B256, EthError> {
        let from = self.signer.address();
        let nonce = self
            .provider
            .get_transaction_count(from)
            .await
            .map_err(|e| EthError::Rpc(e.to_string()))?;

        let mut tx = TxLegacy {
            chain_id: Some(self.chain_id),
            nonce,
            gas_price: 1_000_000_000u128,
            gas_limit: 500_000,
            to: to.into(),
            value: U256::ZERO,
            input: data,
        };
        let sig = self
            .signer
            .sign_transaction_sync(&mut tx)
            .map_err(|e| EthError::Rpc(e.to_string()))?;
        let signed = tx.into_signed(sig);
        let envelope: alloy::consensus::TxEnvelope = signed.into();
        let encoded = alloy::consensus::transaction::TxEnvelope::eip2718_encode(&envelope);

        let pending = self
            .provider
            .send_raw_transaction(&encoded)
            .await
            .map_err(|e| classify_rpc_error(&e.to_string()))?;
        Ok(*pending.tx_hash())
    }

    pub async fn receipt_status_and_memo(
        &self,
        tx_hash: B256,
    ) -> Result<Option<(bool, Option<B256>)>, EthError> {
        let receipt = self
            .provider
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|e| EthError::Rpc(e.to_string()))?;
        let Some(receipt) = receipt else {
            return Ok(None);
        };
        let ok = receipt.status();
        let mut memo = None;
        for log in receipt.inner.logs() {
            if let Ok(decoded) = MemoAnchored::decode_log(log.as_ref(), true) {
                memo = Some(decoded.memoHash);
                break;
            }
        }
        // Fallback: topic0 match if decode_log shape differs
        if memo.is_none() {
            let topic0 = MemoAnchored::SIGNATURE_HASH;
            for log in receipt.inner.logs() {
                if log.topics().first() == Some(&topic0) && log.topics().len() >= 2 {
                    memo = Some(log.topics()[1]);
                    break;
                }
            }
        }
        Ok(Some((ok, memo)))
    }
}

pub fn classify_rpc_error(msg: &str) -> EthError {
    if let Some(reason) = decode_policy_rejected_hex(msg) {
        return EthError::PolicyRejected { reason };
    }
    EthError::Rpc(msg.to_string())
}

/// Decode `PolicyRejected(uint16)` from a hex revert blob or error string.
pub fn decode_policy_rejected_hex(msg: &str) -> Option<u16> {
    // Look for selector + abi-encoded uint16 in the message.
    let selector = hex::encode(PolicyRejected::SELECTOR);
    let lower = msg.to_lowercase();
    if let Some(idx) = lower.find(&selector) {
        let start = idx;
        let hex_chars: String = lower[start..]
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .take(8 + 64)
            .collect();
        if hex_chars.len() >= 8 + 64 {
            let data = hex::decode(&hex_chars).ok()?;
            return decode_policy_rejected_bytes(&data);
        }
    }
    // Try raw 0x… blob
    if let Some(pos) = lower.find("0x") {
        let hex_chars: String = lower[pos + 2..]
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .collect();
        if hex_chars.len() >= 8 {
            let data = hex::decode(&hex_chars).ok()?;
            return decode_policy_rejected_bytes(&data);
        }
    }
    let _ = reason_name; // keep module linked for callers
    None
}

pub fn decode_policy_rejected_bytes(data: &[u8]) -> Option<u16> {
    if data.len() < 4 {
        return None;
    }
    if data[..4] != PolicyRejected::SELECTOR {
        return None;
    }
    let err = PolicyRejected::abi_decode(&data[4..], true).ok()?;
    Some(err.reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_policy_rejected_reason_one() {
        let mut data = PolicyRejected::SELECTOR.to_vec();
        // abi encode uint16(1) as 32-byte word
        let mut word = [0u8; 32];
        word[31] = 1;
        data.extend_from_slice(&word);
        assert_eq!(decode_policy_rejected_bytes(&data), Some(1));
    }

    #[test]
    fn decode_from_error_string() {
        let mut data = PolicyRejected::SELECTOR.to_vec();
        let mut word = [0u8; 32];
        word[31] = 1;
        data.extend_from_slice(&word);
        let msg = format!("execution reverted: 0x{}", hex::encode(&data));
        assert_eq!(decode_policy_rejected_hex(&msg), Some(1));
    }
}
