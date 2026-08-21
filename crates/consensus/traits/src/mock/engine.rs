use crate::block::Block;
use crate::engine::{ConsensusEngine, RunningEngine};
use crate::error::ConsensusError;
use crate::event::{ConsensusEvent, EventSink};
use crate::mock::block::MockBlock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;

/// A mock consensus engine for testing.
///
/// Iterates through pre-supplied blocks, emitting `Finalized` events
/// for each one, then terminates cleanly.
pub struct MockEngine<S: EventSink<Block = MockBlock>> {
    blocks: Vec<MockBlock>,
    sink: Arc<S>,
}

impl<S: EventSink<Block = MockBlock>> MockEngine<S> {
    /// Create a new mock engine that will finalize the given blocks in order.
    pub fn new(blocks: Vec<MockBlock>, sink: Arc<S>) -> Self {
        Self { blocks, sink }
    }
}

impl<S: EventSink<Block = MockBlock>> ConsensusEngine for MockEngine<S> {
    async fn start(self) -> Result<RunningEngine, ConsensusError> {
        let height = Arc::new(AtomicU64::new(0));
        let running = Arc::new(AtomicBool::new(true));

        let h = height.clone();
        let r = running.clone();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        let handle = tokio::spawn(async move {
            for block in self.blocks {
                // Check for shutdown before each block
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }

                let block_height = block.height();
                let event = ConsensusEvent::Finalized {
                    block,
                    height: block_height,
                    proof: vec![],
                };
                self.sink.handle(event).await;
                h.store(block_height, Ordering::Relaxed);
            }

            r.store(false, Ordering::Relaxed);
            Ok(())
        });

        let shutdown = Box::new(move || {
            let _ = shutdown_tx.send(());
        });

        Ok(RunningEngine::new(shutdown, handle, height, running))
    }
}
