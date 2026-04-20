use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, RwLock};

use alloy_consensus::{Transaction, TxReceipt};
use alloy_eips::eip1559::{calc_next_block_base_fee, BaseFeeParams};
use alloy_eips::eip2718::{Decodable2718, Encodable2718};
use alloy_primitives::{bytes::BufMut, Address, Bytes, B256, U256};
use alloy_trie::{root::ordered_trie_root_with_encoder, EMPTY_ROOT_HASH};
use app::{
    decode_extra_data, legacy_proposer_extra_data_bytes,
    traits::{Application, TxSource},
    CanonicalExtraDataV1, EvmBlock, ExecutionResult, ExtraDataDecodeMode, FullDkgV1, Receipt,
};
use evm_precompiles::{
    claimable_balance_slot, community_pool_last_processed_epoch_slot,
    community_pool_locked_remaining_slot, community_pool_unlock_amount_per_cycle_slot,
    community_pool_unlock_every_epochs_slot, current_epoch_slot, COMMUNITY_POOL_ADDRESS,
    EPOCH_PRECOMPILE_ADDRESS, FEE_POOL_PRECOMPILE_ADDRESS,
};
use reth_ethereum_primitives::TransactionSigned;
use reth_evm::{
    execute::{BlockBuilder, BlockExecutor},
    ConfigureEvm, NextBlockEnvAttributes,
};
use reth_primitives_traits::{Header, SealedHeader};
use reth_primitives_traits::{Recovered, SignedTransaction};
use revm::database::states::bundle_state::BundleRetention;
use revm::database::State;
use state::BlockStorage;
use validators::ValidatorEntry;

use crate::canonical_extra_data::{
    build_canonical_extra_data, ensure_full_dkg_players_match_activation,
    full_dkg_should_be_included,
};
use crate::config::WhirlpoolEvmConfig;
use crate::epoch_boundary::{
    apply_boundary_state_to_provider, boundary_required_for_height,
    execute_epoch_boundary_system_call_if_required, load_epoch_boundary_state,
    tx_is_reserved_epoch_namespace, BoundaryCallFailureMode,
};
use crate::error::EvmAppError;
pub use crate::traits::StateProvider;
use crate::validator_activation::{ActivationSourceResolver, BoundaryEpochContext};

pub type RecoveredTx = Recovered<TransactionSigned>;
type ProposedCacheKey = (u64, [u8; 32]);
type ProposedCacheEntry = (ProposedCacheKey, EvmBlock, ExecutionResult, Vec<Receipt>);

#[derive(Clone, Debug)]
struct StagedReceipts {
    height: u64,
    parent_id: [u8; 32],
    block_id: [u8; 32],
    receipts: Vec<Receipt>,
}

#[derive(Clone, Debug)]
pub struct ProposedEvmPayload {
    pub included_user_transactions: Vec<Vec<u8>>,
    pub inclusion_outcomes: Vec<bool>,
    pub result: ExecutionResult,
    pub base_fee_per_gas: u64,
    pub proposer_public_key: [u8; 32],
    pub proposer_fee_recipient: Address,
    pub extra_data: Vec<u8>,
    pub receipts: Vec<Receipt>,
}

include!("header_and_decode.rs");

include!("state_helpers.rs");

#[derive(Clone)]
pub struct EvmApplication<DB> {
    evm_config: WhirlpoolEvmConfig,
    state_db: Arc<RwLock<DB>>,
    tx_source: Arc<dyn TxSource + Send + Sync>,
    pending_receipts: Arc<Mutex<Option<Vec<Receipt>>>>,
    staged_receipts: Arc<Mutex<BTreeMap<[u8; 32], StagedReceipts>>>,
    last_proposed: Arc<Mutex<Option<ProposedCacheEntry>>>,
}

impl<DB> EvmApplication<DB>
where
    DB: std::fmt::Debug,
{
    fn stage_receipts_for_block(&self, block: &EvmBlock, receipts: Vec<Receipt>) {
        {
            let mut guard = self.pending_receipts.lock().unwrap();
            *guard = Some(receipts.clone());
        }

        let staged = StagedReceipts {
            height: block.height,
            parent_id: block.parent_id,
            block_id: block.compute_id(),
            receipts,
        };
        let mut guard = self.staged_receipts.lock().unwrap();
        guard.insert(staged.block_id, staged);
    }
}

mod impl_core_methods;
mod impl_propose;
mod impl_verify;

#[allow(clippy::manual_async_fn)]
impl<DB> Application for EvmApplication<DB>
where
    DB: StateProvider
        + BlockStorage
        + Clone
        + Send
        + Sync
        + 'static
        + revm::Database
        + std::fmt::Debug,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    type Block = EvmBlock;
    type Result = ExecutionResult;
    type Error = EvmAppError;

    fn genesis(&self) -> impl std::future::Future<Output = Self::Block> + Send {
        async move {
            let state_root = {
                let db = self.state_db.read().unwrap();
                db.state_root()
                    .map_err(Into::into)
                    .expect("genesis state root should not fail")
            };
            let genesis_extra_data = build_canonical_extra_data(
                &self.evm_config,
                None,
                self.evm_config.local_proposer_public_key(),
                false,
                0,
            )
            .unwrap_or_else(|_| {
                legacy_proposer_extra_data_bytes(self.evm_config.local_proposer_public_key())
            });

            EvmBlock {
                height: 0,
                parent_id: [0u8; 32],
                state_root: state_root.0,
                transactions_root: EMPTY_ROOT_HASH.0,
                receipts_root: EMPTY_ROOT_HASH.0,
                proposer_public_key: self.evm_config.local_proposer_public_key(),
                proposer_fee_recipient: self.evm_config.fee_recipient().into_array(),
                extra_data: genesis_extra_data,
                gas_used: 0,
                base_fee_per_gas: 1_000_000_000,
                timestamp: 0,
                transactions: vec![],
            }
        }
    }

    fn propose(
        &self,
        parent: &Self::Block,
        height: u64,
    ) -> impl std::future::Future<Output = Result<(Self::Block, Self::Result), Self::Error>> + Send
    {
        async move {
            let parent_id = parent.compute_id();
            let cache_key = (height, parent_id);
            {
                let cache = self.last_proposed.lock().unwrap();
                if let Some((cached_key, ref block, ref result, ref receipts)) = *cache {
                    if cached_key == cache_key {
                        self.stage_receipts_for_block(block, receipts.clone());
                        return Ok((block.clone(), result.clone()));
                    }
                }
            }

            let raw_pending = self.tx_source.pending();
            let timestamp = parent.timestamp + 12;
            let payload = self.propose_evm_transactions(parent, &raw_pending, timestamp, height)?;

            let block_transactions = payload.included_user_transactions.clone();

            let transactions_root =
                ordered_trie_root_with_encoder(&block_transactions, |tx, out| {
                    out.put_slice(tx.as_slice());
                });

            let block = EvmBlock {
                height,
                parent_id,
                state_root: payload.result.state_root,
                transactions_root: transactions_root.0,
                receipts_root: payload.result.receipts_root,
                proposer_public_key: payload.proposer_public_key,
                proposer_fee_recipient: payload.proposer_fee_recipient.into_array(),
                extra_data: payload.extra_data,
                gas_used: payload.result.gas_used,
                base_fee_per_gas: payload.base_fee_per_gas,
                timestamp,
                transactions: block_transactions,
            };
            let receipts = payload.receipts;
            self.stage_receipts_for_block(&block, receipts.clone());

            {
                let mut cache = self.last_proposed.lock().unwrap();
                *cache = Some((cache_key, block.clone(), payload.result.clone(), receipts));
            }

            Ok((block, payload.result))
        }
    }

    fn verify(
        &self,
        parent: &Self::Block,
        block: &Self::Block,
    ) -> impl std::future::Future<Output = Result<Self::Result, Self::Error>> + Send {
        async move {
            let expected_parent_id = parent.compute_id();
            if block.parent_id != expected_parent_id {
                return Err(EvmAppError::InvalidBlock(format!(
                    "Parent id mismatch: expected {:?}, found {:?}",
                    expected_parent_id, block.parent_id
                )));
            }

            let computed_tx_root =
                ordered_trie_root_with_encoder(&block.transactions, |tx, out| out.put_slice(tx));
            if computed_tx_root.0 != block.transactions_root {
                return Err(EvmAppError::InvalidBlock(format!(
                    "Transactions root mismatch: expected {:?}, computed {:?}",
                    block.transactions_root, computed_tx_root.0
                )));
            }

            self.verify_evm_transactions(parent, block, &block.transactions)
        }
    }
}

#[cfg(test)]
mod tests;
