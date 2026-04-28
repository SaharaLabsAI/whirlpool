//! Finalization sink that persists finalized blocks via `BlockStorage` before
//! delegating to the inner `FinalizationSink`.

use app_evm_execution::EvmApplication;
use app_primitives::EvmBlock;
use consensus::event::{ConsensusEvent, EventSink};
use consensus_simplex::FinalizationSink;
use state::{BlockStorage, StateDb};
use std::sync::Arc;
use tracing::{error, info};

/// A finalization sink that persists finalized blocks before delegating to the
/// underlying `FinalizationSink`.
pub struct PersistingFinalizationSink<DB, BS> {
    inner: FinalizationSink<EvmBlock>,
    evm_app: EvmApplication<DB>,
    block_storage: Arc<BS>,
}

impl<DB, BS> PersistingFinalizationSink<DB, BS> {
    pub fn new(
        inner: FinalizationSink<EvmBlock>,
        evm_app: EvmApplication<DB>,
        block_storage: Arc<BS>,
    ) -> Self {
        Self {
            inner,
            evm_app,
            block_storage,
        }
    }
}

impl<DB, BS> EventSink for PersistingFinalizationSink<DB, BS>
where
    DB: StateDb + Send + Sync + 'static + std::fmt::Debug,
    BS: BlockStorage + 'static,
{
    type Block = EvmBlock;

    async fn handle(&self, event: ConsensusEvent<EvmBlock>) {
        if let ConsensusEvent::Finalized { ref block, .. } = event {
            match self
                .evm_app
                .store_finalized_block(block, self.block_storage.as_ref())
            {
                Ok(()) => {
                    info!(
                        height = block.height,
                        "persisted finalized block to storage"
                    );
                }
                Err(e) => {
                    error!(
                        height = block.height,
                        error = %e,
                        "failed to persist finalized block"
                    );
                    // Continue with finalization even if persistence fails.
                    // The block is still finalized by consensus; storage failure
                    // is logged but does not halt the node.
                }
            }
        }

        // Always delegate to inner sink for height tracking + logging
        self.inner.handle(event).await;
    }
}

#[cfg(test)]
mod tests {
    use super::PersistingFinalizationSink;
    use app_evm_execution::{EvmApplication, WhirlpoolEvmConfig};
    use app_evm_state::InMemoryStateDb;
    use app_primitives::{EvmBlock, Receipt};
    use app_traits::traits::TxSource;
    use chainspec::build_sahara_chain_spec;
    use consensus::event::{ConsensusEvent, EventSink};
    use consensus_simplex::FinalizationSink;
    use revm::primitives::B256;
    use state::{BlockStorage, BlockStorageError};
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{Arc, Mutex, RwLock};

    #[derive(Clone)]
    struct EmptyTxSource;

    impl TxSource for EmptyTxSource {
        fn push(&self, _tx: Vec<u8>) {}

        fn pending(&self) -> Vec<Vec<u8>> {
            Vec::new()
        }
    }

    struct MockBlockStorage {
        stored_blocks: Mutex<Vec<(EvmBlock, Vec<Receipt>)>>,
        should_fail: AtomicBool,
    }

    impl MockBlockStorage {
        fn new() -> Self {
            Self {
                stored_blocks: Mutex::new(Vec::new()),
                should_fail: AtomicBool::new(false),
            }
        }

        fn with_failure() -> Self {
            Self {
                stored_blocks: Mutex::new(Vec::new()),
                should_fail: AtomicBool::new(true),
            }
        }

        fn stored_count(&self) -> usize {
            self.stored_blocks.lock().unwrap().len()
        }
    }

    impl BlockStorage for MockBlockStorage {
        fn store_block(
            &self,
            block: &EvmBlock,
            receipts: &[Receipt],
        ) -> Result<(), BlockStorageError> {
            if self.should_fail.load(Ordering::Relaxed) {
                return Err(BlockStorageError::Database("mock failure".into()));
            }
            self.stored_blocks
                .lock()
                .unwrap()
                .push((block.clone(), receipts.to_vec()));
            Ok(())
        }

        fn get_block_by_number(&self, _number: u64) -> Result<Option<EvmBlock>, BlockStorageError> {
            Ok(None)
        }

        fn get_block_by_hash(&self, _hash: B256) -> Result<Option<EvmBlock>, BlockStorageError> {
            Ok(None)
        }

        fn get_receipts_by_block(
            &self,
            _number: u64,
        ) -> Result<Option<Vec<Receipt>>, BlockStorageError> {
            Ok(None)
        }

        fn get_latest_block_number(&self) -> Result<Option<u64>, BlockStorageError> {
            Ok(None)
        }
    }

    fn sample_block(transactions: Vec<Vec<u8>>, height: u64) -> EvmBlock {
        EvmBlock {
            height,
            parent_id: [0u8; 32],
            state_root: [1u8; 32],
            transactions_root: [2u8; 32],
            receipts_root: [3u8; 32],
            proposer_public_key: [0u8; 32],
            proposer_fee_recipient: [0u8; 20],
            extra_data: vec![0u8; 32],
            gas_used: 0,
            base_fee_per_gas: 1,
            timestamp: height,
            transactions,
        }
    }

    fn test_app() -> EvmApplication<InMemoryStateDb> {
        let chain_spec = Arc::new(build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(chain_spec);
        let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));
        let tx_source: Arc<dyn TxSource + Send + Sync> = Arc::new(EmptyTxSource);
        EvmApplication::new(config, state_db, tx_source)
    }

    #[tokio::test]
    async fn prefinalized_events_do_not_store_blocks() {
        let height = Arc::new(AtomicU64::new(0));
        let inner = FinalizationSink::new(height.clone());
        let block_storage = Arc::new(MockBlockStorage::new());
        let sink = PersistingFinalizationSink::new(inner, test_app(), block_storage.clone());
        let block = sample_block(vec![], 1);

        sink.handle(ConsensusEvent::PreFinalized { block, height: 1 })
            .await;

        assert_eq!(block_storage.stored_count(), 0);
        assert_eq!(height.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn finalized_events_persist_blocks() {
        let height = Arc::new(AtomicU64::new(0));
        let inner = FinalizationSink::new(height.clone());
        let block_storage = Arc::new(MockBlockStorage::new());
        let app = test_app();
        let block = sample_block(vec![], 1);
        let sink = PersistingFinalizationSink::new(inner, app, block_storage.clone());

        sink.handle(ConsensusEvent::Finalized {
            block,
            height: 1,
            proof: vec![],
        })
        .await;

        assert_eq!(block_storage.stored_count(), 1);
        assert_eq!(height.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn block_storage_failure_does_not_block_finalization() {
        let height = Arc::new(AtomicU64::new(0));
        let inner = FinalizationSink::new(height.clone());
        let block_storage = Arc::new(MockBlockStorage::with_failure());
        let app = test_app();
        let block = sample_block(vec![], 2);
        let sink = PersistingFinalizationSink::new(inner, app, block_storage);

        sink.handle(ConsensusEvent::Finalized {
            block,
            height: 2,
            proof: vec![],
        })
        .await;

        assert_eq!(height.load(Ordering::SeqCst), 2);
    }
}
