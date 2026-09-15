//! Shared app state and in-memory idempotency store.

use std::collections::HashMap;
use std::sync::Mutex;

use alloy_primitives::B256;

use crate::config::Config;
use crate::eth::EthClient;
use crate::types::PaymentStatus;

#[derive(Clone, Debug)]
pub struct PaymentRecord {
    pub tx_hash: B256,
    pub status: PaymentStatus,
    pub memo_hash: Option<B256>,
}

pub struct AppState {
    pub config: Config,
    pub eth: EthClient,
    /// Only populated after a successful broadcast (tx hash exists).
    payments: Mutex<HashMap<B256, PaymentRecord>>,
}

impl AppState {
    pub fn new(config: Config, eth: EthClient) -> Self {
        Self {
            config,
            eth,
            payments: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_payment(&self, id: &B256) -> Option<PaymentRecord> {
        self.payments.lock().unwrap().get(id).cloned()
    }

    /// Insert only after successful broadcast. Returns existing if already present.
    pub fn insert_if_absent(&self, id: B256, record: PaymentRecord) -> PaymentRecord {
        let mut map = self.payments.lock().unwrap();
        map.entry(id).or_insert(record).clone()
    }

    pub fn update_payment(&self, id: B256, record: PaymentRecord) {
        self.payments.lock().unwrap().insert(id, record);
    }
}
