//! JSON request / response types. Status words: submitted | mined | failed.

use alloy_primitives::{Address, B256, FixedBytes, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PaymentStatus {
    Submitted,
    Mined,
    Failed,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::Mined => "mined",
            Self::Failed => "failed",
        }
    }
}

impl Serialize for PaymentStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub ok: bool,
    pub service: &'static str,
    pub rpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequest {
    pub to: Address,
    /// Decimal string or hex amount.
    #[serde(deserialize_with = "deserialize_u256")]
    pub amount: U256,
    pub end_to_end_id: B256,
    pub uetr: FixedBytes<16>,
    pub instr_id: B256,
    /// ISO currency as ASCII string e.g. "USD", or 0x hex bytes3.
    #[serde(deserialize_with = "deserialize_ccy")]
    pub ccy: FixedBytes<3>,
    pub msg_type: u8,
    pub tr_hash: B256,
    pub pack_hash: B256,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSubmitted {
    pub id: B256,
    pub tx_hash: B256,
    pub status: PaymentStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentView {
    pub id: B256,
    pub tx_hash: B256,
    pub status: PaymentStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo_hash: Option<B256>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreezeRequest {
    pub address: Address,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FreezeResponse {
    pub tx_hash: B256,
    pub address: Address,
    pub status: PaymentStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyRejectedBody {
    pub error: &'static str,
    pub reason: u16,
    pub reason_name: &'static str,
}

fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if let Ok(v) = s.parse::<U256>() {
        return Ok(v);
    }
    // decimal
    U256::from_str_radix(s.trim_start_matches("0x"), if s.starts_with("0x") { 16 } else { 10 })
        .map_err(serde::de::Error::custom)
}

fn deserialize_ccy<'de, D>(deserializer: D) -> Result<FixedBytes<3>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.len() == 3 && s.is_ascii() {
        let b = s.as_bytes();
        return Ok(FixedBytes([b[0], b[1], b[2]]));
    }
    s.parse::<FixedBytes<3>>()
        .map_err(serde::de::Error::custom)
}
