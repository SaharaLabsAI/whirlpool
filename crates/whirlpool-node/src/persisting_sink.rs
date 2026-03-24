//! Composite finalization sink that persists finalized blocks via `BlockStorage`
//! and finalized personality writes via `PersonalityStorage` before delegating
//! to the inner `FinalizationSink`.

use app::EvmBlock;
use app_composite::CompositeApplication;
use app_mem::PersonalityMarkdownTx;
use consensus::event::{ConsensusEvent, EventSink};
use consensus_simplex::FinalizationSink;
use state::{BlockStorage, PersonalityStorage, StateDb, StoredPersonality};
use std::sync::Arc;
use tracing::{error, info};

/// A finalization sink that persists finalized blocks and personality state
/// before delegating to the underlying `FinalizationSink`.
pub struct PersistingFinalizationSink<DB, BS, PS> {
    inner: FinalizationSink<EvmBlock>,
    evm_app: CompositeApplication<DB>,
    block_storage: Arc<BS>,
    personality_storage: Arc<PS>,
}

impl<DB, BS, PS> PersistingFinalizationSink<DB, BS, PS> {
    pub fn new(
        inner: FinalizationSink<EvmBlock>,
        evm_app: CompositeApplication<DB>,
        block_storage: Arc<BS>,
        personality_storage: Arc<PS>,
    ) -> Self {
        Self {
            inner,
            evm_app,
            block_storage,
            personality_storage,
        }
    }

    fn store_finalized_personalities(&self, block: &EvmBlock) -> Result<usize, String>
    where
        PS: PersonalityStorage,
        <PS as PersonalityStorage>::Error: std::fmt::Display,
    {
        let mut persisted = 0usize;

        for raw_tx in &block.transactions {
            let Ok(tx) = PersonalityMarkdownTx::decode(raw_tx) else {
                continue;
            };

            let tx_hash = tx
                .tx_hash()
                .map_err(|err| format!("failed to hash finalized personality tx: {err}"))?;
            let finalized = tx.finalized_write().map_err(|err| {
                format!("failed to derive finalized personality write for block {}: {err}", block.height)
            })?;

            let entry = StoredPersonality {
                tx_hash,
                block_height: block.height,
                signer: finalized.signer,
                personality_id: finalized.personality_id,
                nonce: finalized.nonce,
                markdown: finalized.markdown_bytes,
                markdown_hash: finalized.markdown_hash,
            };

            self.personality_storage
                .put(entry)
                .map_err(|err| format!("failed to persist finalized personality: {err}"))?;
            persisted += 1;
        }

        Ok(persisted)
    }
}

impl<DB, BS, PS> EventSink for PersistingFinalizationSink<DB, BS, PS>
where
    DB: StateDb + Send + Sync + 'static + std::fmt::Debug,
    BS: BlockStorage + 'static,
    PS: PersonalityStorage + 'static,
    <PS as PersonalityStorage>::Error: std::fmt::Display,
{
    type Block = EvmBlock;

    async fn handle(&self, event: ConsensusEvent<EvmBlock>) {
        if let ConsensusEvent::Finalized { ref block, .. } = event {
            match self
                .evm_app
                .store_finalized_block(block, self.block_storage.as_ref())
            {
                Ok(()) => {
                    info!(height = block.height, "persisted finalized block to storage");
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

            match self.store_finalized_personalities(block) {
                Ok(0) => {}
                Ok(count) => {
                    info!(height = block.height, count, "persisted finalized personalities");
                }
                Err(err) => {
                    error!(
                        height = block.height,
                        error = %err,
                        "failed to persist finalized personalities"
                    );
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
    use app::traits::TxSource;
    use app::{EvmBlock, Receipt};
    use app_composite::CompositeApplication;
    use app_evm::{build_sahara_chain_spec, WhirlpoolEvmConfig};
    use app_mem::{PersonalityMarkdownTx, SignatureScheme};
    use consensus::event::{ConsensusEvent, EventSink};
    use consensus_simplex::FinalizationSink;
    use revm::primitives::B256;
    use state::{BlockStorage, BlockStorageError, PersonalityStorage, StoredPersonality};
    use state_memory::InMemoryStateDb;
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
        fn store_block(&self, block: &EvmBlock, receipts: &[Receipt]) -> Result<(), BlockStorageError> {
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

        fn get_receipts_by_block(&self, _number: u64) -> Result<Option<Vec<Receipt>>, BlockStorageError> {
            Ok(None)
        }

        fn get_latest_block_number(&self) -> Result<Option<u64>, BlockStorageError> {
            Ok(None)
        }
    }

    #[derive(Debug)]
    struct MockPersonalityStorageError;

    impl std::fmt::Display for MockPersonalityStorageError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "mock personality storage failure")
        }
    }

    impl std::error::Error for MockPersonalityStorageError {}

    struct MockPersonalityStorage {
        entries: Mutex<Vec<StoredPersonality>>,
        should_fail: AtomicBool,
    }

    impl MockPersonalityStorage {
        fn new() -> Self {
            Self {
                entries: Mutex::new(Vec::new()),
                should_fail: AtomicBool::new(false),
            }
        }

        fn with_failure() -> Self {
            Self {
                entries: Mutex::new(Vec::new()),
                should_fail: AtomicBool::new(true),
            }
        }

        fn stored_entries(&self) -> Vec<StoredPersonality> {
            self.entries.lock().unwrap().clone()
        }
    }

    impl PersonalityStorage for MockPersonalityStorage {
        type Error = MockPersonalityStorageError;

        fn put(&self, entry: StoredPersonality) -> Result<(), Self::Error> {
            if self.should_fail.load(Ordering::Relaxed) {
                return Err(MockPersonalityStorageError);
            }
            self.entries.lock().unwrap().push(entry);
            Ok(())
        }

        fn get_latest(&self, personality_id: &[u8]) -> Result<Option<StoredPersonality>, Self::Error> {
            Ok(self
                .entries
                .lock()
                .unwrap()
                .iter()
                .rev()
                .find(|entry| entry.personality_id == personality_id)
                .cloned())
        }

        fn get_by_signer_nonce(
            &self,
            signer: &[u8],
            nonce: u64,
        ) -> Result<Option<StoredPersonality>, Self::Error> {
            Ok(self
                .entries
                .lock()
                .unwrap()
                .iter()
                .rev()
                .find(|entry| entry.signer == signer && entry.nonce == nonce)
                .cloned())
        }

        fn len(&self) -> Result<usize, Self::Error> {
            Ok(self.entries.lock().unwrap().len())
        }
    }

    fn mem_tx(markdown: &str, nonce: u64) -> Vec<u8> {
        PersonalityMarkdownTx::new(
            b"signer-1".to_vec(),
            b"persona-1".to_vec(),
            nonce,
            markdown.as_bytes().to_vec(),
            SignatureScheme::RawSecp256k1,
            vec![0x11; 65],
        )
        .encode()
        .expect("mem tx encoding must succeed")
    }

    fn sample_block(transactions: Vec<Vec<u8>>, height: u64) -> EvmBlock {
        EvmBlock {
            height,
            parent_id: [0u8; 32],
            state_root: [1u8; 32],
            transactions_root: [2u8; 32],
            receipts_root: [3u8; 32],
            gas_used: 0,
            base_fee_per_gas: 1,
            timestamp: height,
            transactions,
        }
    }

    fn test_app() -> CompositeApplication<InMemoryStateDb> {
        let chain_spec = Arc::new(build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(chain_spec);
        let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));
        let tx_source: Arc<dyn TxSource + Send + Sync> = Arc::new(EmptyTxSource);
        CompositeApplication::new(config, state_db, tx_source)
    }

    #[tokio::test]
    async fn prefinalized_events_do_not_make_personality_visible() {
        let height = Arc::new(AtomicU64::new(0));
        let inner = FinalizationSink::new(height.clone());
        let block_storage = Arc::new(MockBlockStorage::new());
        let personality_storage = Arc::new(MockPersonalityStorage::new());
        let sink = PersistingFinalizationSink::new(
            inner,
            test_app(),
            block_storage.clone(),
            personality_storage.clone(),
        );
        let block = sample_block(vec![mem_tx("# Draft", 1)], 1);

        sink.handle(ConsensusEvent::PreFinalized { block, height: 1 }).await;

        assert_eq!(block_storage.stored_count(), 0);
        assert!(personality_storage.stored_entries().is_empty());
        assert_eq!(height.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn finalized_events_persist_personality_writes() {
        let height = Arc::new(AtomicU64::new(0));
        let inner = FinalizationSink::new(height.clone());
        let block_storage = Arc::new(MockBlockStorage::new());
        let personality_storage = Arc::new(MockPersonalityStorage::new());
        let sink = PersistingFinalizationSink::new(
            inner,
            test_app(),
            block_storage.clone(),
            personality_storage.clone(),
        );
        let block = sample_block(vec![mem_tx("# Final", 7)], 7);

        sink.handle(ConsensusEvent::Finalized {
            block,
            height: 7,
            proof: vec![],
        })
        .await;

        assert_eq!(block_storage.stored_count(), 1);
        let stored = personality_storage.stored_entries();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].block_height, 7);
        assert_eq!(stored[0].nonce, 7);
        assert_eq!(stored[0].markdown, b"# Final".to_vec());
        assert_eq!(height.load(Ordering::SeqCst), 7);
    }

    #[tokio::test]
    async fn personality_storage_failure_does_not_block_finalization() {
        let height = Arc::new(AtomicU64::new(0));
        let inner = FinalizationSink::new(height.clone());
        let block_storage = Arc::new(MockBlockStorage::new());
        let personality_storage = Arc::new(MockPersonalityStorage::with_failure());
        let sink = PersistingFinalizationSink::new(
            inner,
            test_app(),
            block_storage.clone(),
            personality_storage,
        );
        let block = sample_block(vec![mem_tx("# Final", 3)], 3);

        sink.handle(ConsensusEvent::Finalized {
            block,
            height: 3,
            proof: vec![],
        })
        .await;

        assert_eq!(block_storage.stored_count(), 1);
        assert_eq!(height.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn block_storage_failure_does_not_block_personality_flush() {
        let height = Arc::new(AtomicU64::new(0));
        let inner = FinalizationSink::new(height.clone());
        let block_storage = Arc::new(MockBlockStorage::with_failure());
        let personality_storage = Arc::new(MockPersonalityStorage::new());
        let sink = PersistingFinalizationSink::new(
            inner,
            test_app(),
            block_storage,
            personality_storage.clone(),
        );
        let block = sample_block(vec![mem_tx("# Final", 4)], 4);

        sink.handle(ConsensusEvent::Finalized {
            block,
            height: 4,
            proof: vec![],
        })
        .await;

        let stored = personality_storage.stored_entries();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].nonce, 4);
        assert_eq!(height.load(Ordering::SeqCst), 4);
    }
}
