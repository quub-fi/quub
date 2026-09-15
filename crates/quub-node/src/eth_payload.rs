//! `--dev` payload builder: lane-ordered BestTransactions into stock ethereum payload.

use quub_payload::{order_lane_candidates, LaneCandidate};
use quub_pool::is_payment_bytes;
use quub_primitives::PAYMENT_LANE_BPS;
use reth_basic_payload_builder::{BuildArguments, BuildOutcome, PayloadBuilder, PayloadConfig};
use reth_ethereum::{
    chainspec::{ChainSpecProvider, EthereumHardforks},
    evm::primitives::{ConfigureEvm, NextBlockEnvAttributes},
    pool::{
        error::InvalidPoolTransactionError, BestTransactions, BestTransactionsAttributes,
        PoolTransaction, TransactionPool, ValidPoolTransaction,
    },
    primitives::NodePrimitives,
    storage::StateProviderFactory,
    EthPrimitives,
};
use reth_ethereum_engine_primitives::{EthBuiltPayload, EthPayloadAttributes};
use reth_ethereum_payload_builder::{
    default_ethereum_payload, EthereumBuilderConfig,
    EthereumPayloadBuilder as StockEthPayloadBuilder,
};
use reth_payload_builder_primitives::PayloadBuilderError;
use std::{collections::VecDeque, sync::Arc};

// alloy_consensus::Transaction provides `.input()` on pooled txs (via PoolTransaction).
use alloy_consensus::Transaction;

type BestTxIter<Pool> = Box<
    dyn BestTransactions<Item = Arc<ValidPoolTransaction<<Pool as TransactionPool>::Transaction>>>,
>;

/// Builds ethereum payloads with Quub 70/30 lane ordering (ADR-017).
#[derive(Debug, Clone)]
pub struct QuubEthPayloadBuilder<Pool, Client, Evm> {
    inner: StockEthPayloadBuilder<Pool, Client, Evm>,
    pool: Pool,
    client: Client,
    evm_config: Evm,
    builder_config: EthereumBuilderConfig,
}

impl<Pool: Clone, Client: Clone, Evm: Clone> QuubEthPayloadBuilder<Pool, Client, Evm> {
    pub fn new(
        client: Client,
        pool: Pool,
        evm_config: Evm,
        builder_config: EthereumBuilderConfig,
    ) -> Self {
        Self {
            inner: StockEthPayloadBuilder::new(
                client.clone(),
                pool.clone(),
                evm_config.clone(),
                builder_config.clone(),
            ),
            pool,
            client,
            evm_config,
            builder_config,
        }
    }
}

fn lane_ordered_best_txs<Pool>(pool: &Pool, attr: BestTransactionsAttributes) -> BestTxIter<Pool>
where
    Pool: TransactionPool,
    Pool::Transaction: PoolTransaction,
{
    let _ = PAYMENT_LANE_BPS;
    let block_gas = pool.block_info().block_gas_limit.max(1);
    let pending = pool.pending_transactions();

    let mut indexed: Vec<(
        usize,
        LaneCandidate<usize>,
        Arc<ValidPoolTransaction<Pool::Transaction>>,
    )> = Vec::with_capacity(pending.len());
    for (i, vtx) in pending.into_iter().enumerate() {
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
            vtx,
        ));
    }

    let candidates: Vec<_> = indexed.iter().map(|(_, c, _)| c.clone()).collect();
    let order = order_lane_candidates(block_gas, candidates);
    let mut q = VecDeque::new();
    for id in order {
        if let Some((_, _, vtx)) = indexed.iter().find(|(i, _, _)| *i == id) {
            q.push_back(vtx.clone());
        }
    }
    Box::new(LaneBestTransactions {
        queue: q,
        skip_blobs: false,
    })
}

struct LaneBestTransactions<T: PoolTransaction> {
    queue: VecDeque<Arc<ValidPoolTransaction<T>>>,
    skip_blobs: bool,
}

impl<T: PoolTransaction> Iterator for LaneBestTransactions<T> {
    type Item = Arc<ValidPoolTransaction<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(tx) = self.queue.pop_front() {
            if self.skip_blobs && tx.is_eip4844() {
                continue;
            }
            return Some(tx);
        }
        None
    }
}

impl<T: PoolTransaction> BestTransactions for LaneBestTransactions<T> {
    fn mark_invalid(&mut self, tx: &Self::Item, _kind: InvalidPoolTransactionError) {
        let sender = *tx.sender_ref();
        let nonce = tx.nonce();
        self.queue
            .retain(|t| !(t.sender_ref() == &sender && t.nonce() >= nonce));
    }

    fn no_updates(&mut self) {}

    fn set_skip_blobs(&mut self, skip_blobs: bool) {
        self.skip_blobs = skip_blobs;
    }
}

impl<Pool, Client, Evm> PayloadBuilder for QuubEthPayloadBuilder<Pool, Client, Evm>
where
    Evm: ConfigureEvm<Primitives = EthPrimitives, NextBlockEnvCtx = NextBlockEnvAttributes>
        + Clone,
    Client: StateProviderFactory + ChainSpecProvider<ChainSpec: EthereumHardforks> + Clone,
    Pool: TransactionPool<
            Transaction: PoolTransaction<Consensus = <EthPrimitives as NodePrimitives>::SignedTx>,
        > + Clone,
{
    type Attributes = EthPayloadAttributes;
    type BuiltPayload = EthBuiltPayload;

    fn try_build(
        &self,
        args: BuildArguments<EthPayloadAttributes, EthBuiltPayload>,
    ) -> Result<BuildOutcome<EthBuiltPayload>, PayloadBuilderError> {
        let pool = self.pool.clone();
        default_ethereum_payload(
            self.evm_config.clone(),
            self.client.clone(),
            pool.clone(),
            self.builder_config.clone(),
            args,
            |attributes| lane_ordered_best_txs(&pool, attributes),
        )
    }

    fn build_empty_payload(
        &self,
        config: PayloadConfig<Self::Attributes>,
    ) -> Result<EthBuiltPayload, PayloadBuilderError> {
        self.inner.build_empty_payload(config)
    }
}
