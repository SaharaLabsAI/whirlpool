use std::{
    ops::{Bound, RangeBounds},
    sync::Arc,
};

use alloy_consensus::{BlockBody, Header};
use alloy_primitives::{BlockNumber, TxNumber};
use app_evm_state::RethStateDb;
use reth_chain_state::{
    CanonStateNotification, CanonStateNotifications, CanonStateSubscriptions,
    ForkChoiceNotifications, ForkChoiceSubscriptions, PersistedBlockNotifications,
    PersistedBlockSubscriptions,
};
use reth_chainspec::ChainSpec;
use reth_ethereum_primitives::{Block, EthPrimitives};
use reth_primitives_traits::{Block as _, RecoveredBlock, SealedHeader};
use reth_storage_api::BlockHashReader;
use reth_storage_errors::provider::{ProviderError, ProviderResult};
use tokio::sync::{broadcast, watch};

fn map_db_err(e: impl std::fmt::Display) -> ProviderError {
    ProviderError::Database(reth_storage_errors::db::DatabaseError::Other(e.to_string()))
}

fn range_to_exclusive_bounds(range: impl RangeBounds<BlockNumber>) -> (BlockNumber, BlockNumber) {
    let start = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(&start) => start.saturating_add(1),
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&end) => end.saturating_add(1),
        Bound::Excluded(&end) => end,
        Bound::Unbounded => u64::MAX,
    };
    (start, end)
}

fn tx_range_to_exclusive_bounds(range: impl RangeBounds<TxNumber>) -> (TxNumber, TxNumber) {
    let start = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(&start) => start.saturating_add(1),
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&end) => end.saturating_add(1),
        Bound::Excluded(&end) => end,
        Bound::Unbounded => u64::MAX,
    };
    (start, end)
}

#[derive(Clone, Debug)]
pub struct WhirlpoolProvider {
    state_db: Arc<RethStateDb>,
    chain_spec: Arc<ChainSpec>,
    canon_state_tx: broadcast::Sender<CanonStateNotification<EthPrimitives>>,
    safe_block_tx: watch::Sender<Option<SealedHeader<Header>>>,
    finalized_block_tx: watch::Sender<Option<SealedHeader<Header>>>,
    persisted_block_tx: watch::Sender<Option<alloy_eips::BlockNumHash>>,
}

impl WhirlpoolProvider {
    pub fn new(state_db: Arc<RethStateDb>, chain_spec: Arc<ChainSpec>) -> Self {
        let (canon_state_tx, _rx) = broadcast::channel(16);
        let (safe_block_tx, _safe_block_rx) = watch::channel(None);
        let (finalized_block_tx, _finalized_block_rx) = watch::channel(None);
        let (persisted_block_tx, _persisted_block_rx) = watch::channel(None);
        Self {
            state_db,
            chain_spec,
            canon_state_tx,
            safe_block_tx,
            finalized_block_tx,
            persisted_block_tx,
        }
    }

    pub fn state_db(&self) -> &Arc<RethStateDb> {
        &self.state_db
    }
}

impl WhirlpoolProvider {
    fn read_block_by_number(&self, number: BlockNumber) -> ProviderResult<Option<Block>> {
        let Some(block) = self
            .state_db
            .rpc_reader()
            .blocks()
            .bodies()
            .read_block_by_number(number)
            .map_err(map_db_err)?
        else {
            return Ok(None);
        };

        Ok(Some(Block::new(
            block.header,
            BlockBody {
                transactions: block.transactions,
                ommers: vec![],
                withdrawals: None,
            },
        )))
    }

    fn recovered_block_by_number(
        &self,
        number: BlockNumber,
        sealed: bool,
    ) -> ProviderResult<Option<RecoveredBlock<Block>>> {
        let Some(block) = self.read_block_by_number(number)? else {
            return Ok(None);
        };

        if sealed {
            let Some(hash) = self.block_hash(number)? else {
                return Ok(None);
            };
            return block
                .seal_unchecked(hash)
                .try_recover()
                .map(Some)
                .map_err(|_| ProviderError::SenderRecoveryError);
        }

        block
            .try_into_recovered()
            .map(Some)
            .map_err(|_| ProviderError::SenderRecoveryError)
    }
}

mod block;
mod state;
mod subscriptions;
mod transactions;
