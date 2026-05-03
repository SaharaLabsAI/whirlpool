use std::{ops::RangeBounds, sync::Arc};

use ::state::StateDb as StateDbTrait;
use alloy_eips::BlockNumberOrTag;
use alloy_primitives::{
    Address, BlockHash, BlockNumber, Bytes, StorageKey, StorageValue, B256, U256,
};
use reth_chainspec::{ChainSpec, ChainSpecProvider};
use reth_db_api::models::AccountBeforeTx;
use reth_ethereum_primitives::Receipt;
use reth_execution_types::ExecutionOutcome;
use reth_primitives_traits::{Account, Bytecode};
use reth_prune::{PruneCheckpoint, PruneSegment};
use reth_stages_api::{StageCheckpoint, StageId};
use reth_storage_api::{
    AccountReader, BlockIdReader, BlockNumReader, BytecodeReader, ChangeSetReader,
    HashedPostStateProvider, PruneCheckpointReader, StageCheckpointReader, StateProofProvider,
    StateProvider, StateProviderBox, StateProviderFactory, StateReader, StateRootProvider,
    StorageRootProvider,
};
use reth_storage_errors::provider::{ProviderError, ProviderResult};
use reth_trie::{
    updates::TrieUpdates, AccountProof, HashedPostState, HashedStorage, MultiProof,
    MultiProofTargets, StorageMultiProof, StorageProof, TrieInput,
};

use crate::provider_impl::{map_db_err, WhirlpoolProvider};

impl AccountReader for WhirlpoolProvider {
    fn basic_account(&self, address: &Address) -> ProviderResult<Option<Account>> {
        self.state_db
            .rpc_reader()
            .basic_account(*address)
            .map_err(map_db_err)
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
        account: Address,
        storage_key: StorageKey,
    ) -> ProviderResult<Option<StorageValue>> {
        let index = U256::from_be_bytes(storage_key.0);
        let value =
            StateDbTrait::get_storage(&*self.state_db, account, index).map_err(map_db_err)?;
        if value.is_zero() {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }
}

impl BytecodeReader for WhirlpoolProvider {
    fn bytecode_by_hash(&self, code_hash: &B256) -> ProviderResult<Option<Bytecode>> {
        let bytecode =
            StateDbTrait::get_code_by_hash(&*self.state_db, *code_hash).map_err(map_db_err)?;
        if bytecode.bytes().is_empty() {
            Ok(None)
        } else {
            Ok(Some(Bytecode(bytecode)))
        }
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
