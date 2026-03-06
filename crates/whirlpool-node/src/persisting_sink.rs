//! Composite finalization sink that persists blocks via `BlockStorage`
//! before delegating to the inner `FinalizationSink`.

use app::EvmBlock;
use app_evm::executor::EvmApplication;
use consensus::event::{ConsensusEvent, EventSink};
use consensus_simplex::FinalizationSink;
use state::{BlockStorage, StateDb};
use std::sync::Arc;
use tracing::{error, info};

/// A finalization sink that persists finalized blocks to storage
/// before delegating to the underlying `FinalizationSink`.
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

impl<DB: StateDb + Send + Sync + 'static, BS: BlockStorage + 'static> EventSink
    for PersistingFinalizationSink<DB, BS>
{
    type Block = EvmBlock;

    async fn handle(&self, event: ConsensusEvent<EvmBlock>) {
        // If finalized, persist the block before delegating
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
        }

        // Always delegate to inner sink for height tracking + logging
        self.inner.handle(event).await;
    }
}
