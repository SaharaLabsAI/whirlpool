use crate::block::EmptyBlock;
use consensus_core::{ConsensusEvent, EventSink};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

/// FinalizationSink tracks finalized block heights and logs consensus events.
///
/// This sink implements EventSink<Block=EmptyBlock> and updates an atomic height
/// counter when blocks are finalized. It provides visibility into consensus progress
/// via structured logging.
pub struct FinalizationSink {
    height: Arc<AtomicU64>,
}

impl FinalizationSink {
    /// Creates a new FinalizationSink with shared height tracking.
    ///
    /// # Arguments
    /// * `height` - Arc to AtomicU64 for thread-safe height updates
    pub fn new(height: Arc<AtomicU64>) -> Self {
        Self { height }
    }

    /// Returns the current finalized height.
    pub fn current_height(&self) -> u64 {
        self.height.load(Ordering::SeqCst)
    }
}

impl EventSink for FinalizationSink {
    type Block = EmptyBlock;

    async fn handle(&self, event: ConsensusEvent<EmptyBlock>) {
        use consensus_core::Block as CoreBlock;

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
    use crate::block::EmptyBlock;
    use consensus_core::{ConsensusEvent, EventSink};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_handle_finalized_logs_height() {
        let height = Arc::new(AtomicU64::new(0));
        let sink = super::FinalizationSink::new(height.clone());
        let block = EmptyBlock::new(1, [0; 32]);
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
        let sink = super::FinalizationSink::new(height.clone());
        let block = EmptyBlock::new(5, [0; 32]);
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
        let sink = super::FinalizationSink::new(height.clone());
        let block = EmptyBlock::new(3, [0; 32]);
        sink.handle(ConsensusEvent::PreFinalized { block, height: 3 })
            .await;
        assert_eq!(height.load(Ordering::SeqCst), 0); // unchanged
    }

    #[tokio::test]
    async fn test_handle_fault_logs_warning() {
        let height = Arc::new(AtomicU64::new(0));
        let sink = super::FinalizationSink::new(height.clone());
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
        let sink = super::FinalizationSink::new(height.clone());
        for h in 1..=3 {
            let block = EmptyBlock::new(h, [0; 32]);
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
        let sink = super::FinalizationSink::new(height.clone());
        assert_eq!(sink.current_height(), 0);
    }
}
