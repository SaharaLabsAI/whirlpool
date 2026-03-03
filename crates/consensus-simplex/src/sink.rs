use consensus::block::Block;
use consensus::event::{ConsensusEvent, EventSink};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

/// FinalizationSink tracks finalized block heights and logs consensus events.
///
/// This sink implements EventSink for any block type B: Block and updates an atomic height
/// counter when blocks are finalized. It provides visibility into consensus progress
/// via structured logging.
#[derive(Clone)]
pub struct FinalizationSink<B: Block> {
    height: Arc<AtomicU64>,
    _phantom: PhantomData<B>,
}

impl<B: Block> FinalizationSink<B> {
    /// Creates a new FinalizationSink with shared height tracking.
    ///
    /// # Arguments
    /// * `height` - Arc to AtomicU64 for thread-safe height updates
    pub fn new(height: Arc<AtomicU64>) -> Self {
        Self {
            height,
            _phantom: PhantomData,
        }
    }

    /// Returns the current finalized height.
    pub fn current_height(&self) -> u64 {
        self.height.load(Ordering::SeqCst)
    }
}

impl<B: Block> EventSink for FinalizationSink<B> {
    type Block = B;

    async fn handle(&self, event: ConsensusEvent<B>) {
        use consensus::block::Block as CoreBlock;

        match event {
            ConsensusEvent::Finalized {
                block,
                height,
                proof: _,
            } => {
                self.height.store(height, Ordering::SeqCst);
                info!(height = height, block_id = ?CoreBlock::id(&block), "block finalized");
            }
            ConsensusEvent::PreFinalized { block: _, height } => {
                info!(height = height, "block pre-finalized");
            }
            ConsensusEvent::Fault {
                offender,
                evidence: _,
            } => {
                warn!(?offender, "consensus fault detected");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestBlock;
    use consensus::event::ConsensusEvent;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_handle_finalized_logs_height() {
        let height = Arc::new(AtomicU64::new(0));
        let sink = FinalizationSink::<TestBlock>::new(height.clone());
        let block = TestBlock::child(&TestBlock::genesis());
        sink.handle(ConsensusEvent::Finalized {
            block,
            height: 1,
            proof: vec![],
        })
        .await;
        // Test passes if no panic
    }

    #[tokio::test]
    async fn test_handle_finalized_updates_atomic_height() {
        let height = Arc::new(AtomicU64::new(0));
        let sink = FinalizationSink::<TestBlock>::new(height.clone());
        let genesis = TestBlock::genesis();
        let block = TestBlock::child(&genesis);
        sink.handle(ConsensusEvent::Finalized {
            block,
            height: 5,
            proof: vec![],
        })
        .await;
        assert_eq!(height.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_handle_prefinalized_is_noop() {
        let height = Arc::new(AtomicU64::new(0));
        let sink = FinalizationSink::<TestBlock>::new(height.clone());
        let genesis = TestBlock::genesis();
        let block = TestBlock::child(&genesis);
        sink.handle(ConsensusEvent::PreFinalized { block, height: 3 })
            .await;
        assert_eq!(height.load(Ordering::SeqCst), 0); // unchanged
    }

    #[tokio::test]
    async fn test_handle_fault_logs_warning() {
        let height = Arc::new(AtomicU64::new(0));
        let sink = FinalizationSink::<TestBlock>::new(height.clone());
        sink.handle(ConsensusEvent::Fault {
            offender: vec![1, 2, 3],
            evidence: vec![],
        })
        .await;
        // Test passes if no panic
    }

    #[tokio::test]
    async fn test_height_monotonically_increases() {
        let height = Arc::new(AtomicU64::new(0));
        let sink = FinalizationSink::<TestBlock>::new(height.clone());
        let genesis = TestBlock::genesis();
        for h in 1..=3 {
            let block = TestBlock::child(&genesis);
            sink.handle(ConsensusEvent::Finalized {
                block,
                height: h,
                proof: vec![],
            })
            .await;
        }
        assert_eq!(height.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_initial_height_is_zero() {
        let height = Arc::new(AtomicU64::new(0));
        let sink = FinalizationSink::<TestBlock>::new(height.clone());
        assert_eq!(sink.current_height(), 0);
    }
}
