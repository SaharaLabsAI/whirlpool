use std::{
    ops::{Bound, RangeBounds, RangeInclusive},
    sync::Arc,
};

use ::state::StateDb as StateDbTrait;
use alloy_consensus::{transaction::TransactionMeta, BlockBody, BlockHeader, Header};
use alloy_eips::{BlockHashOrNumber, BlockId, BlockNumberOrTag};
use alloy_primitives::{
    Address, BlockHash, BlockNumber, Bytes, StorageKey, StorageValue, TxHash, TxNumber, B256, U256,
};
use reth_chain_state::{
    CanonStateNotification, CanonStateNotifications, CanonStateSubscriptions,
    ForkChoiceNotifications, ForkChoiceSubscriptions, PersistedBlockNotifications,
    PersistedBlockSubscriptions,
};
use reth_chainspec::{ChainInfo, ChainSpec, ChainSpecProvider};
use reth_db_api::{
    cursor::DbCursorRO,
    models::{AccountBeforeTx, StoredBlockBodyIndices},
    transaction::DbTx,
    Database,
};
use reth_ethereum_primitives::{Block, EthPrimitives, Receipt, TransactionSigned};
use reth_execution_types::ExecutionOutcome;
use reth_primitives_traits::{
    Account, Block as _, Bytecode, RecoveredBlock, SealedHeader, SignerRecoverable,
};
use reth_prune::{PruneCheckpoint, PruneSegment};
use reth_stages_api::{StageCheckpoint, StageId};
use reth_storage_api::{
    AccountReader, BlockBodyIndicesProvider, BlockHashReader, BlockIdReader, BlockNumReader,
    BlockReader, BlockReaderIdExt, BlockSource, BytecodeReader, ChangeSetReader,
    HashedPostStateProvider, HeaderProvider, NodePrimitivesProvider, PruneCheckpointReader,
    ReceiptProvider, ReceiptProviderIdExt, StageCheckpointReader, StateProofProvider,
    StateProvider, StateProviderBox, StateProviderFactory, StateReader, StateRootProvider,
    StorageRootProvider, TransactionVariant, TransactionsProvider,
};
use reth_storage_errors::provider::{ProviderError, ProviderResult};
use reth_trie::{
    updates::TrieUpdates, AccountProof, HashedPostState, HashedStorage, MultiProof,
    MultiProofTargets, StorageMultiProof, StorageProof, TrieInput,
};
use state_reth::{
    tables::{
        BlockBodyIndices, CanonicalHeaders, HeaderNumbers, Headers, PlainAccountState, Receipts,
        TransactionBlocks, TransactionHashNumbers, Transactions,
    },
    RethStateDb,
};
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
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let Some(header) = tx.get::<Headers>(number).map_err(map_db_err)? else {
            return Ok(None);
        };
        let body_indices = tx
            .get::<BlockBodyIndices>(number)
            .map_err(map_db_err)?
            .unwrap_or_default();

        let mut transactions = Vec::with_capacity(body_indices.tx_count as usize);
        for tx_num in body_indices.tx_num_range() {
            let Some(transaction) = tx.get::<Transactions>(tx_num).map_err(map_db_err)? else {
                return Ok(None);
            };
            transactions.push(transaction);
        }

        Ok(Some(Block::new(
            header,
            BlockBody {
                transactions,
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
