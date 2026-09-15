//! eth_* JSON-RPC client + legacy tx signing (Alloy primitives 0.8 line).

use alloy_primitives::{keccak256, Address, Bytes, B256, U256};
use alloy_sol_types::{sol, SolCall, SolError};
use k256::ecdsa::{signature::hazmat::PrehashSigner, RecoveryId, Signature, SigningKey, VerifyingKey};
use quub_primitives::{CHAIN_ID_TESTNET, PAYMENT_TOKEN, POLICY_ADMIN};
use serde_json::{json, Value};

use crate::config::Config;

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

#[derive(Clone)]
pub struct EthClient {
    http: reqwest::Client,
    rpc_url: String,
    signing_key: SigningKey,
    operator_address: Address,
    chain_id: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum EthError {
    #[error("rpc: {0}")]
    Rpc(String),
    #[error("policy rejected reason={reason}")]
    PolicyRejected { reason: u16 },
    #[allow(dead_code)]
    #[error("tx failed: {0}")]
    TxFailed(String),
}

impl EthClient {
    pub async fn connect(config: &Config) -> Result<Self, String> {
        Ok(Self {
            http: reqwest::Client::new(),
            rpc_url: config.rpc_url.clone(),
            signing_key: config.signing_key.clone(),
            operator_address: config.operator_address,
            chain_id: config.chain_id,
        })
    }

    pub fn operator_address(&self) -> Address {
        self.operator_address
    }

    pub async fn chain_id(&self) -> Result<u64, EthError> {
        let hex: String = self.rpc("eth_chainId", json!([])).await?;
        parse_u64_hex(&hex).map_err(EthError::Rpc)
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
        let params = json!([{
            "from": format!("{:?}", self.operator_address),
            "to": format!("{to:?}"),
            "data": format!("0x{}", hex::encode(&data)),
        }, "latest"]);
        match self.rpc::<Value>("eth_call", params).await {
            Ok(_) => Ok(()),
            Err(EthError::Rpc(msg)) => Err(classify_rpc_error(&msg)),
            Err(e) => Err(e),
        }
    }

    async fn send_legacy(&self, to: Address, data: Bytes) -> Result<B256, EthError> {
        let nonce_hex: String = self
            .rpc(
                "eth_getTransactionCount",
                json!([format!("{:?}", self.operator_address), "pending"]),
            )
            .await?;
        let nonce = parse_u64_hex(&nonce_hex).map_err(EthError::Rpc)?;

        let signed = sign_legacy_tx(
            &self.signing_key,
            self.chain_id,
            nonce,
            1_000_000_000u128,
            500_000,
            to,
            U256::ZERO,
            &data,
        )
        .map_err(EthError::Rpc)?;

        let raw = format!("0x{}", hex::encode(&signed));
        let tx_hash: String = self
            .rpc("eth_sendRawTransaction", json!([raw]))
            .await
            .map_err(|e| match e {
                EthError::Rpc(msg) => classify_rpc_error(&msg),
                other => other,
            })?;
        tx_hash
            .parse::<B256>()
            .map_err(|e| EthError::Rpc(format!("bad tx hash: {e}")))
    }

    pub async fn receipt_status_and_memo(
        &self,
        tx_hash: B256,
    ) -> Result<Option<(bool, Option<B256>)>, EthError> {
        let receipt: Option<Value> = self
            .rpc(
                "eth_getTransactionReceipt",
                json!([format!("{tx_hash:?}")]),
            )
            .await?;
        let Some(receipt) = receipt else {
            return Ok(None);
        };
        let status_hex = receipt
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");
        let ok = parse_u64_hex(status_hex).unwrap_or(0) == 1;
        let mut memo = None;
        let topic0 = format!(
            "{:?}",
            alloy_primitives::keccak256(b"MemoAnchored(bytes32,uint8)")
        )
        .to_lowercase();
        if let Some(logs) = receipt.get("logs").and_then(|v| v.as_array()) {
            for log in logs {
                let topics = log
                    .get("topics")
                    .and_then(|t| t.as_array())
                    .cloned()
                    .unwrap_or_default();
                let first = topics
                    .first()
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                if first == topic0 {
                    if let Some(h) = topics.get(1).and_then(|t| t.as_str()) {
                        if let Ok(m) = h.parse::<B256>() {
                            memo = Some(m);
                            break;
                        }
                    }
                }
            }
        }
        Ok(Some((ok, memo)))
    }

    async fn rpc<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, EthError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let resp = self
            .http
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| EthError::Rpc(e.to_string()))?;
        let v: Value = resp
            .json()
            .await
            .map_err(|e| EthError::Rpc(e.to_string()))?;
        if let Some(err) = v.get("error") {
            return Err(EthError::Rpc(err.to_string()));
        }
        let result = v
            .get("result")
            .cloned()
            .ok_or_else(|| EthError::Rpc("missing result".into()))?;
        serde_json::from_value(result).map_err(|e| EthError::Rpc(e.to_string()))
    }
}

fn parse_u64_hex(s: &str) -> Result<u64, String> {
    let s = s.trim().trim_start_matches("0x");
    u64::from_str_radix(if s.is_empty() { "0" } else { s }, 16)
        .map_err(|e| format!("bad hex u64: {e}"))
}

fn encode_legacy_list(fields: &[RlpItem<'_>]) -> Vec<u8> {
    let mut payload = Vec::new();
    for f in fields {
        f.encode(&mut payload);
    }
    let mut out = Vec::new();
    if payload.len() <= 55 {
        out.push(0xc0 + payload.len() as u8);
        out.extend_from_slice(&payload);
    } else {
        let len_be = payload.len().to_be_bytes();
        let len_bytes = trim_be(&len_be);
        out.push(0xf7 + len_bytes.len() as u8);
        out.extend_from_slice(len_bytes);
        out.extend_from_slice(&payload);
    }
    out
}

enum RlpItem<'a> {
    U64(u64),
    U128(u128),
    U256(U256),
    Addr(Address),
    Bytes(&'a [u8]),
}

impl RlpItem<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            RlpItem::U64(v) => encode_u64(*v, out),
            RlpItem::U128(v) => encode_u128(*v, out),
            RlpItem::U256(v) => {
                let be = v.to_be_bytes::<32>();
                encode_bytes(trim_be(&be), out);
            }
            RlpItem::Addr(a) => encode_bytes(a.as_slice(), out),
            RlpItem::Bytes(b) => encode_bytes(b, out),
        }
    }
}

fn encode_u64(v: u64, out: &mut Vec<u8>) {
    if v == 0 {
        out.push(0x80);
        return;
    }
    let be = v.to_be_bytes();
    encode_bytes(trim_be(&be), out);
}

fn encode_u128(v: u128, out: &mut Vec<u8>) {
    if v == 0 {
        out.push(0x80);
        return;
    }
    let be = v.to_be_bytes();
    encode_bytes(trim_be(&be), out);
}

fn encode_bytes(b: &[u8], out: &mut Vec<u8>) {
    if b.len() == 1 && b[0] < 0x80 {
        out.push(b[0]);
        return;
    }
    if b.len() <= 55 {
        out.push(0x80 + b.len() as u8);
        out.extend_from_slice(b);
    } else {
        let len_be = b.len().to_be_bytes();
        let len_bytes = trim_be(&len_be);
        out.push(0xb7 + len_bytes.len() as u8);
        out.extend_from_slice(len_bytes);
        out.extend_from_slice(b);
    }
}

fn trim_be(be: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < be.len() && be[i] == 0 {
        i += 1;
    }
    if i == be.len() {
        &be[be.len()..be.len()]
    } else {
        &be[i..]
    }
}

/// EIP-155 legacy signed tx bytes.
fn sign_legacy_tx(
    key: &SigningKey,
    chain_id: u64,
    nonce: u64,
    gas_price: u128,
    gas_limit: u64,
    to: Address,
    value: U256,
    data: &[u8],
) -> Result<Vec<u8>, String> {
    let unsigned = encode_legacy_list(&[
        RlpItem::U64(nonce),
        RlpItem::U128(gas_price),
        RlpItem::U64(gas_limit),
        RlpItem::Addr(to),
        RlpItem::U256(value),
        RlpItem::Bytes(data),
        RlpItem::U64(chain_id),
        RlpItem::U64(0),
        RlpItem::U64(0),
    ]);
    let hash = keccak256(&unsigned);

    let sig: Signature = key
        .sign_prehash(hash.as_slice())
        .map_err(|e| format!("sign: {e}"))?;
    let (recovery_id, sig_bytes) = recover_v_rs(key, &hash, &sig)?;

    let v = (chain_id * 2 + 35) + u64::from(recovery_id);
    let r = U256::from_be_slice(&sig_bytes[..32]);
    let s = U256::from_be_slice(&sig_bytes[32..64]);

    Ok(encode_legacy_list(&[
        RlpItem::U64(nonce),
        RlpItem::U128(gas_price),
        RlpItem::U64(gas_limit),
        RlpItem::Addr(to),
        RlpItem::U256(value),
        RlpItem::Bytes(data),
        RlpItem::U64(v),
        RlpItem::U256(r),
        RlpItem::U256(s),
    ]))
}

fn recover_v_rs(
    key: &SigningKey,
    hash: &B256,
    sig: &Signature,
) -> Result<(u8, [u8; 64]), String> {
    let sig_bytes = sig.to_bytes();
    let mut out = [0u8; 64];
    out.copy_from_slice(&sig_bytes);

    for recid in [0u8, 1u8] {
        let rid = RecoveryId::from_byte(recid).ok_or("bad recovery id")?;
        if let Ok(recovered) = VerifyingKey::recover_from_prehash(hash.as_slice(), sig, rid) {
            if recovered == *key.verifying_key() {
                return Ok((recid, out));
            }
        }
    }
    Err("could not determine recovery id".into())
}

pub fn classify_rpc_error(msg: &str) -> EthError {
    if let Some(reason) = decode_policy_rejected_hex(msg) {
        return EthError::PolicyRejected { reason };
    }
    EthError::Rpc(msg.to_string())
}

/// Decode `PolicyRejected(uint16)` from a hex revert blob or error string.
pub fn decode_policy_rejected_hex(msg: &str) -> Option<u16> {
    let selector = hex::encode(PolicyRejected::SELECTOR);
    let lower = msg.to_lowercase();
    if let Some(idx) = lower.find(&selector) {
        let hex_chars: String = lower[idx..]
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .take(8 + 64)
            .collect();
        if hex_chars.len() >= 8 + 64 {
            let data = hex::decode(&hex_chars).ok()?;
            return decode_policy_rejected_bytes(&data);
        }
    }
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
    None
}

pub fn decode_policy_rejected_bytes(data: &[u8]) -> Option<u16> {
    if data.len() < 4 {
        return None;
    }
    if data[..4] != PolicyRejected::SELECTOR {
        return None;
    }
    let err = PolicyRejected::abi_decode(data, true).ok()?;
    Some(err.reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_policy_rejected_reason_one() {
        let mut data = PolicyRejected::SELECTOR.to_vec();
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
