//! Mode A: lane-ordered pool candidates for stock `OpPayloadBuilder`.

use quub_payload::{order_lane_candidates, LaneCandidate};
use quub_pool::is_payment_bytes;
use quub_primitives::PAYMENT_LANE_BPS;
use reth_optimism_payload_builder::builder::OpPayloadTransactions;
use reth_optimism_txpool::OpPooledTx;
use reth_payload_util::{PayloadTransactions, PayloadTransactionsFixed};
use reth_transaction_pool::{BestTransactionsAttributes, PoolTransaction, TransactionPool};
use std::marker::PhantomData;

/// Reorders pending pool txs via [`quub_payload::fill_lanes`], then hands them to stock OP builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct QuubLaneTxs {
    _pd: PhantomData<()>,
}

impl QuubLaneTxs {
    pub const fn new() -> Self {
        Self { _pd: PhantomData }
    }
}

impl<T> OpPayloadTransactions<T> for QuubLaneTxs
where
    T: PoolTransaction + OpPooledTx,
{
    fn best_transactions<Pool: TransactionPool<Transaction = T>>(
        &self,
        pool: Pool,
        attr: BestTransactionsAttributes,
    ) -> impl PayloadTransactions<Transaction = T> {
        let _ = PAYMENT_LANE_BPS;
        let block_gas = pool.block_info().block_gas_limit.max(1);
        let pending = pool.pending_transactions();

        let mut indexed: Vec<(usize, LaneCandidate<usize>, T)> = Vec::with_capacity(pending.len());
        for (i, vtx) in pending.iter().enumerate() {
            let to = vtx.to().unwrap_or_default();
            let input = vtx.transaction.input();
            let payment = is_payment_bytes(to.as_slice(), input.as_ref());
            let tip = vtx
                .effective_tip_per_gas(attr.basefee)
                .unwrap_or_else(|| vtx.priority_fee_or_price());
            indexed.push((
                i,
                LaneCandidate {
                    id: i,
                    is_payment: payment,
                    gas_limit: vtx.gas_limit(),
                    tip,
                },
                vtx.transaction.clone(),
            ));
        }

        let candidates: Vec<_> = indexed.iter().map(|(_, c, _)| c.clone()).collect();
        let order = order_lane_candidates(block_gas, candidates);
        let txs: Vec<T> = order
            .into_iter()
            .filter_map(|id| indexed.iter().find(|(i, _, _)| *i == id).map(|(_, _, tx)| tx.clone()))
            .collect();

        PayloadTransactionsFixed::new(txs)
    }
}
