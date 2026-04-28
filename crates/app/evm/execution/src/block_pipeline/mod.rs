use std::sync::{Arc, Mutex, RwLock};

use alloy_consensus::Transaction;
use alloy_eips::eip1559::{calc_next_block_base_fee, BaseFeeParams};
use alloy_primitives::{bytes::BufMut, Address, TxKind};
use alloy_trie::{root::ordered_trie_root_with_encoder, EMPTY_ROOT_HASH};
use app_primitives::{
    header_extra_data::build_raw_eth_envelope, EvmBlock, ExecutionResult, Receipt,
};
use app_traits::traits::{Application, TxSource};
use evm_precompiles::{
    reserved_advance_epoch_call_matches, EpochBoundaryRuntimeError, PostBlockAccountingRuntimeError,
};
use reth_ethereum_primitives::TransactionSigned;
use reth_primitives_traits::SealedHeader;
use state::BlockStorage;

pub use crate::codec::RecoveredTx;
use crate::config::WhirlpoolEvmConfig;
use crate::error::EvmAppError;
use crate::post_handle::ReceiptStore;
pub use crate::traits::StateDb;
use validators_dkg::{DkgHistory, DkgMetadataError};

mod propose;
mod state_helpers;
mod verify;

type ProposedCacheKey = (u64, [u8; 32]);
type ProposedCacheEntry = (ProposedCacheKey, EvmBlock, ExecutionResult, Vec<Receipt>);
const BLOCK_GAS_LIMIT: u64 = 30_000_000;

enum TxExecutionErrorDisposition {
    InvalidTxValidation(String),
    OtherValidation(String),
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundaryCallFailureMode {
    Propose,
    Verify,
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

#[derive(Clone)]
pub struct EvmApplication<DB> {
    evm_config: WhirlpoolEvmConfig,
    state_db: Arc<RwLock<DB>>,
    tx_source: Arc<dyn TxSource + Send + Sync>,
    receipt_store: ReceiptStore,
    last_proposed: Arc<Mutex<Option<ProposedCacheEntry>>>,
}

impl<DB> EvmApplication<DB>
where
    DB: std::fmt::Debug,
{
    pub fn new(
        evm_config: WhirlpoolEvmConfig,
        state_db: Arc<RwLock<DB>>,
        tx_source: Arc<dyn TxSource + Send + Sync>,
    ) -> Self {
        Self {
            evm_config,
            state_db,
            tx_source,
            receipt_store: ReceiptStore::new(),
            last_proposed: Arc::new(Mutex::new(None)),
        }
    }

    pub fn store_finalized_block(
        &self,
        block: &EvmBlock,
        storage: &dyn BlockStorage,
    ) -> Result<(), EvmAppError> {
        self.receipt_store.store_finalized_block(block, storage)
    }

    #[cfg(test)]
    fn has_staged_receipts_for(&self, block_id: [u8; 32]) -> bool {
        self.receipt_store.has_staged_receipts_for(block_id)
    }

    #[cfg(test)]
    fn staged_receipts_is_empty(&self) -> bool {
        self.receipt_store.staged_receipts_is_empty()
    }

    pub fn pending_receipts(&self) -> Vec<Receipt> {
        self.receipt_store.pending_receipts()
    }
}

fn tx_is_reserved_epoch_namespace(tx: &TransactionSigned, signer: Address) -> bool {
    match tx.kind() {
        TxKind::Call(target_address) => {
            reserved_advance_epoch_call_matches(signer, target_address, tx.value(), tx.input())
        }
        _ => false,
    }
}

fn map_epoch_boundary_runtime_error(
    err: EpochBoundaryRuntimeError,
    failure_mode: BoundaryCallFailureMode,
) -> EvmAppError {
    match err {
        EpochBoundaryRuntimeError::StateAccess(message) => EvmAppError::State(message),
        EpochBoundaryRuntimeError::InvalidStoredValue(message) => {
            EvmAppError::InvalidBlock(message.into())
        }
        EpochBoundaryRuntimeError::SystemCallExecution(message) => {
            boundary_call_failure(failure_mode, message)
        }
        EpochBoundaryRuntimeError::SystemCallUnsuccessful => boundary_call_failure(
            failure_mode,
            "required epoch boundary system call did not succeed".into(),
        ),
        EpochBoundaryRuntimeError::EffectExtraction(message) => {
            boundary_call_failure(failure_mode, message)
        }
    }
}

fn boundary_call_failure(mode: BoundaryCallFailureMode, message: String) -> EvmAppError {
    match mode {
        BoundaryCallFailureMode::Propose => EvmAppError::Execution(message),
        BoundaryCallFailureMode::Verify => EvmAppError::InvalidBlock(message),
    }
}

fn map_dkg_metadata_error(err: DkgMetadataError) -> EvmAppError {
    match err {
        DkgMetadataError::History(message) => EvmAppError::State(message),
        other => EvmAppError::InvalidBlock(other.to_string()),
    }
}

fn map_post_block_accounting_runtime_error(err: PostBlockAccountingRuntimeError) -> EvmAppError {
    match err {
        PostBlockAccountingRuntimeError::StateAccess(message) => EvmAppError::State(message),
        PostBlockAccountingRuntimeError::InvalidStoredValue(message) => {
            EvmAppError::InvalidBlock(message)
        }
        PostBlockAccountingRuntimeError::Execution(message) => EvmAppError::Execution(message),
    }
}

fn classify_tx_execution_error(
    err: reth_evm::execute::BlockExecutionError,
) -> TxExecutionErrorDisposition {
    match err {
        reth_evm::execute::BlockExecutionError::Validation(
            reth_evm::execute::BlockValidationError::InvalidTx { .. },
        ) => TxExecutionErrorDisposition::InvalidTxValidation(err.to_string()),
        reth_evm::execute::BlockExecutionError::Validation(other) => {
            TxExecutionErrorDisposition::OtherValidation(other.to_string())
        }
        other => TxExecutionErrorDisposition::Other(other.to_string()),
    }
}

fn expected_next_block_base_fee(parent: &EvmBlock) -> u64 {
    calc_next_block_base_fee(
        parent.gas_used,
        BLOCK_GAS_LIMIT,
        parent.base_fee_per_gas,
        BaseFeeParams::ethereum(),
    )
}

fn build_sealed_header(block: &EvmBlock) -> SealedHeader {
    let header = crate::codec::build_header_from_evm_block(block);
    let hash = header.hash_slow();
    SealedHeader::new(header, hash)
}

#[allow(clippy::manual_async_fn)]
impl<DB> Application for EvmApplication<DB>
where
    DB: StateDb + DkgHistory + Clone + Send + Sync + 'static + revm::Database + std::fmt::Debug,
    <DB as StateDb>::Error: Into<EvmAppError>,
    <DB as DkgHistory>::Error: std::fmt::Display,
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
            let genesis_extra_data =
                build_raw_eth_envelope(self.evm_config.local_proposer_public_key())
                    .expect("genesis raw_eth extra_data envelope should encode");

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
                        self.receipt_store.stage_for_block(block, receipts.clone());
                        return Ok((block.clone(), result.clone()));
                    }
                }
            }

            let raw_pending = crate::ingress::pending_transactions(self.tx_source.as_ref());
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
            self.receipt_store.stage_for_block(&block, receipts.clone());

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

            let block_transactions = crate::ingress::candidate_block_transactions(block);
            let computed_tx_root =
                ordered_trie_root_with_encoder(block_transactions, |tx, out| out.put_slice(tx));
            if computed_tx_root.0 != block.transactions_root {
                return Err(EvmAppError::InvalidBlock(format!(
                    "Transactions root mismatch: expected {:?}, computed {:?}",
                    block.transactions_root, computed_tx_root.0
                )));
            }

            self.verify_evm_transactions(parent, block, block_transactions)
        }
    }
}

#[cfg(test)]
mod tests;
