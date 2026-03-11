use std::{
    ops::{Bound, RangeBounds, RangeInclusive},
    sync::Arc,
};

use alloy_consensus::{transaction::TransactionMeta, BlockBody, Header};
use alloy_eips::{BlockHashOrNumber, BlockId, BlockNumberOrTag};
use alloy_primitives::{
    Address, BlockHash, BlockNumber, Bytes, StorageKey, StorageValue, TxHash, TxNumber, B256,
};
use reth_chain_state::{CanonStateNotification, CanonStateNotifications, CanonStateSubscriptions};
use reth_chainspec::{ChainInfo, ChainSpec, ChainSpecProvider};
use reth_db_api::{
    cursor::DbCursorRO,
    models::{AccountBeforeTx, StoredBlockBodyIndices},
    transaction::DbTx,
    Database,
};
use reth_ethereum_primitives::{Block, EthPrimitives, Receipt, TransactionSigned};
use reth_execution_types::ExecutionOutcome;
use reth_primitives_traits::{Account, Block as _, Bytecode, RecoveredBlock, SealedHeader};
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
        BlockBodyIndices, CanonicalHeaders, HeaderNumbers, Headers, TransactionBlocks, Transactions,
    },
    RethStateDb,
};
use tokio::sync::broadcast;

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

#[derive(Clone, Debug)]
pub struct WhirlpoolProvider {
    state_db: Arc<RethStateDb>,
    chain_spec: Arc<ChainSpec>,
    canon_state_tx: broadcast::Sender<CanonStateNotification<EthPrimitives>>,
}

impl WhirlpoolProvider {
    pub fn new(state_db: Arc<RethStateDb>, chain_spec: Arc<ChainSpec>) -> Self {
        let (canon_state_tx, _rx) = broadcast::channel(16);
        Self {
            state_db,
            chain_spec,
            canon_state_tx,
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

impl NodePrimitivesProvider for WhirlpoolProvider {
    type Primitives = EthPrimitives;
}

impl BlockHashReader for WhirlpoolProvider {
    fn block_hash(&self, number: u64) -> ProviderResult<Option<B256>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        tx.get::<CanonicalHeaders>(number).map_err(map_db_err)
    }

    fn canonical_hashes_range(
        &self,
        start: BlockNumber,
        end: BlockNumber,
    ) -> ProviderResult<Vec<B256>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let mut hashes = Vec::new();
        for number in start..end {
            if let Some(hash) = tx.get::<CanonicalHeaders>(number).map_err(map_db_err)? {
                hashes.push(hash);
            }
        }
        Ok(hashes)
    }
}

impl BlockNumReader for WhirlpoolProvider {
    fn chain_info(&self) -> ProviderResult<ChainInfo> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let best_number = tx
            .cursor_read::<CanonicalHeaders>()
            .map_err(map_db_err)?
            .last()
            .map_err(map_db_err)?
            .map(|(number, _)| number)
            .unwrap_or(0);
        let best_hash = tx
            .get::<CanonicalHeaders>(best_number)
            .map_err(map_db_err)?
            .unwrap_or_default();
        Ok(ChainInfo {
            best_hash,
            best_number,
        })
    }

    fn best_block_number(&self) -> ProviderResult<BlockNumber> {
        Ok(self.chain_info()?.best_number)
    }

    fn last_block_number(&self) -> ProviderResult<BlockNumber> {
        self.best_block_number()
    }

    fn block_number(&self, hash: B256) -> ProviderResult<Option<BlockNumber>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        tx.get::<HeaderNumbers>(hash).map_err(map_db_err)
    }
}

impl BlockIdReader for WhirlpoolProvider {
    fn pending_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        Ok(None)
    }

    fn safe_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        Ok(None)
    }

    fn finalized_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        Ok(None)
    }
}

impl HeaderProvider for WhirlpoolProvider {
    type Header = Header;

    fn header(&self, block_hash: BlockHash) -> ProviderResult<Option<Self::Header>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let Some(number) = tx.get::<HeaderNumbers>(block_hash).map_err(map_db_err)? else {
            return Ok(None);
        };
        tx.get::<Headers>(number).map_err(map_db_err)
    }

    fn header_by_number(&self, num: u64) -> ProviderResult<Option<Self::Header>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        tx.get::<Headers>(num).map_err(map_db_err)
    }

    fn headers_range(
        &self,
        range: impl RangeBounds<BlockNumber>,
    ) -> ProviderResult<Vec<Self::Header>> {
        let (start, end) = range_to_exclusive_bounds(range);
        if start >= end {
            return Ok(Vec::new());
        }

        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let mut headers = Vec::new();
        for number in start..end {
            if let Some(header) = tx.get::<Headers>(number).map_err(map_db_err)? {
                headers.push(header);
            }
        }
        Ok(headers)
    }

    fn sealed_header(
        &self,
        number: BlockNumber,
    ) -> ProviderResult<Option<SealedHeader<Self::Header>>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let Some(header) = tx.get::<Headers>(number).map_err(map_db_err)? else {
            return Ok(None);
        };
        let Some(hash) = tx.get::<CanonicalHeaders>(number).map_err(map_db_err)? else {
            return Ok(None);
        };
        Ok(Some(SealedHeader::new(header, hash)))
    }

    fn sealed_headers_while(
        &self,
        range: impl RangeBounds<BlockNumber>,
        mut predicate: impl FnMut(&SealedHeader<Self::Header>) -> bool,
    ) -> ProviderResult<Vec<SealedHeader<Self::Header>>> {
        let (start, end) = range_to_exclusive_bounds(range);
        if start >= end {
            return Ok(Vec::new());
        }

        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let mut headers = Vec::new();
        for number in start..end {
            let Some(header) = tx.get::<Headers>(number).map_err(map_db_err)? else {
                continue;
            };
            let Some(hash) = tx.get::<CanonicalHeaders>(number).map_err(map_db_err)? else {
                continue;
            };
            let sealed = SealedHeader::new(header, hash);
            if !predicate(&sealed) {
                break;
            }
            headers.push(sealed);
        }
        Ok(headers)
    }
}

impl BlockReader for WhirlpoolProvider {
    type Block = Block;

    fn find_block_by_hash(
        &self,
        hash: B256,
        _source: BlockSource,
    ) -> ProviderResult<Option<Self::Block>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let Some(number) = tx.get::<HeaderNumbers>(hash).map_err(map_db_err)? else {
            return Ok(None);
        };
        self.block(BlockHashOrNumber::Number(number))
    }

    fn block(&self, id: BlockHashOrNumber) -> ProviderResult<Option<Self::Block>> {
        let number = match id {
            BlockHashOrNumber::Hash(hash) => {
                let tx = self.state_db.inner().tx().map_err(map_db_err)?;
                tx.get::<HeaderNumbers>(hash).map_err(map_db_err)?
            }
            BlockHashOrNumber::Number(number) => Some(number),
        };

        let Some(number) = number else {
            return Ok(None);
        };
        self.read_block_by_number(number)
    }

    fn pending_block(&self) -> ProviderResult<Option<RecoveredBlock<Self::Block>>> {
        Ok(None)
    }

    fn pending_block_and_receipts(
        &self,
    ) -> ProviderResult<Option<(RecoveredBlock<Self::Block>, Vec<Self::Receipt>)>> {
        Ok(None)
    }

    fn recovered_block(
        &self,
        id: BlockHashOrNumber,
        _transaction_kind: TransactionVariant,
    ) -> ProviderResult<Option<RecoveredBlock<Self::Block>>> {
        let Some(number) = self.convert_hash_or_number(id)? else {
            return Ok(None);
        };
        self.recovered_block_by_number(number, false)
    }

    fn sealed_block_with_senders(
        &self,
        id: BlockHashOrNumber,
        _transaction_kind: TransactionVariant,
    ) -> ProviderResult<Option<RecoveredBlock<Self::Block>>> {
        let Some(number) = self.convert_hash_or_number(id)? else {
            return Ok(None);
        };
        self.recovered_block_by_number(number, true)
    }

    fn block_range(&self, range: RangeInclusive<BlockNumber>) -> ProviderResult<Vec<Self::Block>> {
        let mut blocks = Vec::new();
        for number in range {
            if let Some(block) = self.read_block_by_number(number)? {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    fn block_with_senders_range(
        &self,
        range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<Vec<RecoveredBlock<Self::Block>>> {
        let mut blocks = Vec::new();
        for number in range {
            if let Some(block) = self.recovered_block_by_number(number, false)? {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    fn recovered_block_range(
        &self,
        range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<Vec<RecoveredBlock<Self::Block>>> {
        let mut blocks = Vec::new();
        for number in range {
            if let Some(block) = self.recovered_block_by_number(number, true)? {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    fn block_by_transaction_id(&self, id: TxNumber) -> ProviderResult<Option<BlockNumber>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let mut cursor = tx.cursor_read::<TransactionBlocks>().map_err(map_db_err)?;
        let entry = cursor.seek(id).map_err(map_db_err)?;
        Ok(entry.map(|(_, block_number)| block_number))
    }
}

impl BlockReaderIdExt for WhirlpoolProvider {
    fn block_by_id(&self, _id: BlockId) -> ProviderResult<Option<Self::Block>> {
        Ok(None)
    }

    fn sealed_header_by_id(
        &self,
        _id: BlockId,
    ) -> ProviderResult<Option<SealedHeader<Self::Header>>> {
        Ok(None)
    }

    fn header_by_id(&self, _id: BlockId) -> ProviderResult<Option<Self::Header>> {
        Ok(None)
    }
}

impl TransactionsProvider for WhirlpoolProvider {
    type Transaction = TransactionSigned;

    fn transaction_id(&self, _tx_hash: TxHash) -> ProviderResult<Option<TxNumber>> {
        Ok(None)
    }

    fn transaction_by_id(&self, _id: TxNumber) -> ProviderResult<Option<Self::Transaction>> {
        Ok(None)
    }

    fn transaction_by_id_unhashed(
        &self,
        _id: TxNumber,
    ) -> ProviderResult<Option<Self::Transaction>> {
        Ok(None)
    }

    fn transaction_by_hash(&self, _hash: TxHash) -> ProviderResult<Option<Self::Transaction>> {
        Ok(None)
    }

    fn transaction_by_hash_with_meta(
        &self,
        _hash: TxHash,
    ) -> ProviderResult<Option<(Self::Transaction, TransactionMeta)>> {
        Ok(None)
    }

    fn transactions_by_block(
        &self,
        _block_id: BlockHashOrNumber,
    ) -> ProviderResult<Option<Vec<Self::Transaction>>> {
        Ok(None)
    }

    fn transactions_by_block_range(
        &self,
        _range: impl RangeBounds<BlockNumber>,
    ) -> ProviderResult<Vec<Vec<Self::Transaction>>> {
        Ok(Vec::default())
    }

    fn transactions_by_tx_range(
        &self,
        _range: impl RangeBounds<TxNumber>,
    ) -> ProviderResult<Vec<Self::Transaction>> {
        Ok(Vec::default())
    }

    fn senders_by_tx_range(
        &self,
        _range: impl RangeBounds<TxNumber>,
    ) -> ProviderResult<Vec<Address>> {
        Ok(Vec::default())
    }

    fn transaction_sender(&self, _id: TxNumber) -> ProviderResult<Option<Address>> {
        Ok(None)
    }
}

impl ReceiptProvider for WhirlpoolProvider {
    type Receipt = Receipt;

    fn receipt(&self, _id: TxNumber) -> ProviderResult<Option<Self::Receipt>> {
        Ok(None)
    }

    fn receipt_by_hash(&self, _hash: TxHash) -> ProviderResult<Option<Self::Receipt>> {
        Ok(None)
    }

    fn receipts_by_block(
        &self,
        _block: BlockHashOrNumber,
    ) -> ProviderResult<Option<Vec<Self::Receipt>>> {
        Ok(None)
    }

    fn receipts_by_tx_range(
        &self,
        _range: impl RangeBounds<TxNumber>,
    ) -> ProviderResult<Vec<Self::Receipt>> {
        Ok(Vec::new())
    }

    fn receipts_by_block_range(
        &self,
        _block_range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<Vec<Vec<Self::Receipt>>> {
        Ok(Vec::new())
    }
}

impl ReceiptProviderIdExt for WhirlpoolProvider {}

impl AccountReader for WhirlpoolProvider {
    fn basic_account(&self, _address: &Address) -> ProviderResult<Option<Account>> {
        Ok(None)
    }
}

impl ChainSpecProvider for WhirlpoolProvider {
    type ChainSpec = ChainSpec;

    fn chain_spec(&self) -> Arc<Self::ChainSpec> {
        self.chain_spec.clone()
    }
}

impl StateRootProvider for WhirlpoolProvider {
    fn state_root(&self, _state: HashedPostState) -> ProviderResult<B256> {
        Ok(B256::default())
    }

    fn state_root_from_nodes(&self, _input: TrieInput) -> ProviderResult<B256> {
        Ok(B256::default())
    }

    fn state_root_with_updates(
        &self,
        _state: HashedPostState,
    ) -> ProviderResult<(B256, TrieUpdates)> {
        Ok((B256::default(), TrieUpdates::default()))
    }

    fn state_root_from_nodes_with_updates(
        &self,
        _input: TrieInput,
    ) -> ProviderResult<(B256, TrieUpdates)> {
        Ok((B256::default(), TrieUpdates::default()))
    }
}

impl StorageRootProvider for WhirlpoolProvider {
    fn storage_root(
        &self,
        _address: Address,
        _hashed_storage: HashedStorage,
    ) -> ProviderResult<B256> {
        Ok(B256::default())
    }

    fn storage_proof(
        &self,
        _address: Address,
        slot: B256,
        _hashed_storage: HashedStorage,
    ) -> ProviderResult<StorageProof> {
        Ok(StorageProof::new(slot))
    }

    fn storage_multiproof(
        &self,
        _address: Address,
        _slots: &[B256],
        _hashed_storage: HashedStorage,
    ) -> ProviderResult<StorageMultiProof> {
        Ok(StorageMultiProof::empty())
    }
}

impl StateProofProvider for WhirlpoolProvider {
    fn proof(
        &self,
        _input: TrieInput,
        address: Address,
        _slots: &[B256],
    ) -> ProviderResult<AccountProof> {
        Ok(AccountProof::new(address))
    }

    fn multiproof(
        &self,
        _input: TrieInput,
        _targets: MultiProofTargets,
    ) -> ProviderResult<MultiProof> {
        Ok(MultiProof::default())
    }

    fn witness(&self, _input: TrieInput, _target: HashedPostState) -> ProviderResult<Vec<Bytes>> {
        Ok(Vec::default())
    }
}

impl HashedPostStateProvider for WhirlpoolProvider {
    fn hashed_post_state(&self, _bundle_state: &revm_database::BundleState) -> HashedPostState {
        HashedPostState::default()
    }
}

impl StateReader for WhirlpoolProvider {
    type Receipt = Receipt;

    fn get_state(
        &self,
        _block: BlockNumber,
    ) -> ProviderResult<Option<ExecutionOutcome<Self::Receipt>>> {
        Ok(None)
    }
}

impl StateProvider for WhirlpoolProvider {
    fn storage(
        &self,
        _account: Address,
        _storage_key: StorageKey,
    ) -> ProviderResult<Option<StorageValue>> {
        Ok(None)
    }
}

impl BytecodeReader for WhirlpoolProvider {
    fn bytecode_by_hash(&self, _code_hash: &B256) -> ProviderResult<Option<Bytecode>> {
        Ok(None)
    }
}

impl StateProviderFactory for WhirlpoolProvider {
    fn latest(&self) -> ProviderResult<StateProviderBox> {
        Ok(Box::new(self.clone()))
    }

    fn state_by_block_number_or_tag(
        &self,
        number_or_tag: BlockNumberOrTag,
    ) -> ProviderResult<StateProviderBox> {
        match number_or_tag {
            BlockNumberOrTag::Latest => self.latest(),
            BlockNumberOrTag::Finalized => {
                let hash = self
                    .finalized_block_hash()?
                    .ok_or(ProviderError::FinalizedBlockNotFound)?;
                self.history_by_block_hash(hash)
            }
            BlockNumberOrTag::Safe => {
                let hash = self
                    .safe_block_hash()?
                    .ok_or(ProviderError::SafeBlockNotFound)?;
                self.history_by_block_hash(hash)
            }
            BlockNumberOrTag::Earliest => {
                self.history_by_block_number(self.earliest_block_number()?)
            }
            BlockNumberOrTag::Pending => self.pending(),
            BlockNumberOrTag::Number(num) => self.history_by_block_number(num),
        }
    }

    fn history_by_block_number(&self, _block: BlockNumber) -> ProviderResult<StateProviderBox> {
        Ok(Box::new(self.clone()))
    }

    fn history_by_block_hash(&self, _block: BlockHash) -> ProviderResult<StateProviderBox> {
        Ok(Box::new(self.clone()))
    }

    fn state_by_block_hash(&self, _block: BlockHash) -> ProviderResult<StateProviderBox> {
        Ok(Box::new(self.clone()))
    }

    fn pending(&self) -> ProviderResult<StateProviderBox> {
        Ok(Box::new(self.clone()))
    }

    fn pending_state_by_hash(&self, _block_hash: B256) -> ProviderResult<Option<StateProviderBox>> {
        Ok(Some(Box::new(self.clone())))
    }

    fn maybe_pending(&self) -> ProviderResult<Option<StateProviderBox>> {
        Ok(Some(Box::new(self.clone())))
    }
}

impl CanonStateSubscriptions for WhirlpoolProvider {
    fn subscribe_to_canonical_state(&self) -> CanonStateNotifications<Self::Primitives> {
        self.canon_state_tx.subscribe()
    }
}

impl StageCheckpointReader for WhirlpoolProvider {
    fn get_stage_checkpoint(&self, _id: StageId) -> ProviderResult<Option<StageCheckpoint>> {
        Ok(None)
    }

    fn get_stage_checkpoint_progress(&self, _id: StageId) -> ProviderResult<Option<Vec<u8>>> {
        Ok(None)
    }

    fn get_all_checkpoints(&self) -> ProviderResult<Vec<(String, StageCheckpoint)>> {
        Ok(Vec::new())
    }
}

impl ChangeSetReader for WhirlpoolProvider {
    fn account_block_changeset(
        &self,
        _block_number: BlockNumber,
    ) -> ProviderResult<Vec<AccountBeforeTx>> {
        Ok(Vec::default())
    }

    fn get_account_before_block(
        &self,
        _block_number: BlockNumber,
        _address: Address,
    ) -> ProviderResult<Option<AccountBeforeTx>> {
        Ok(None)
    }

    fn account_changesets_range(
        &self,
        _range: impl RangeBounds<BlockNumber>,
    ) -> ProviderResult<Vec<(BlockNumber, AccountBeforeTx)>> {
        Ok(Vec::default())
    }

    fn account_changeset_count(&self) -> ProviderResult<usize> {
        Ok(0)
    }
}

impl PruneCheckpointReader for WhirlpoolProvider {
    fn get_prune_checkpoint(
        &self,
        _segment: PruneSegment,
    ) -> ProviderResult<Option<PruneCheckpoint>> {
        Ok(None)
    }

    fn get_prune_checkpoints(&self) -> ProviderResult<Vec<(PruneSegment, PruneCheckpoint)>> {
        Ok(Vec::new())
    }
}

impl BlockBodyIndicesProvider for WhirlpoolProvider {
    fn block_body_indices(&self, num: u64) -> ProviderResult<Option<StoredBlockBodyIndices>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        tx.get::<BlockBodyIndices>(num).map_err(map_db_err)
    }

    fn block_body_indices_range(
        &self,
        range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<Vec<StoredBlockBodyIndices>> {
        let tx = self.state_db.inner().tx().map_err(map_db_err)?;
        let mut indices = Vec::new();
        for number in range {
            if let Some(body_indices) = tx.get::<BlockBodyIndices>(number).map_err(map_db_err)? {
                indices.push(body_indices);
            }
        }
        Ok(indices)
    }
}
