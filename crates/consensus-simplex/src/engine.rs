// CommonwareEngine — sealed wiring for commonware simplex BFT consensus
//!
//! This module provides sealed internal wiring that connects ConsensusApp and EventSink
//! to the commonware simplex BFT engine. All infrastructure (Mailbox, MailboxActor,
//! AppAdapter, FinalizationSink) is created and managed internally.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures::channel::mpsc;
use tokio::task::JoinHandle;

use consensus::app::ConsensusApp;
use consensus::block::Block as CoreBlock;
use consensus::engine::{ConsensusEngine, RunningEngine};
use consensus::error::ConsensusError;
use consensus::event::EventSink;

use crate::config::CommonwareConfig;
use crate::mailbox::{Mailbox, MailboxActor};
use crate::sink::FinalizationSink;
use crate::types::CommonwareBlock;

/// A consensus engine backed by the Commonware Simplex BFT protocol.
///
/// `CommonwareEngine` provides sealed internal wiring that connects your application
/// and event sink to the simplex consensus engine. All infrastructure components
/// (mailbox bridging, actor spawning, adapter wiring) are handled internally.
///
/// # Construction
///
/// ```ignore
/// let engine = CommonwareEngine::new(app, sink, config);
/// let running = engine.start()?;
/// ```
///
/// # Stub Implementation
///
/// **CURRENT STATUS:** This is a STUB implementation that simulates consensus by
/// incrementing block height every 5 seconds. Full simplex engine wiring (P2P,
/// marshal actor, simplex::Engine) is future work pending P2P configuration design.
pub struct CommonwareEngine<A, S>
where
    A: ConsensusApp,
    S: EventSink<Block = A::Block>,
    A::Block: CommonwareBlock,
{
    app: Arc<A>,
    sink: Arc<S>,
    config: CommonwareConfig,
}

impl<A, S> CommonwareEngine<A, S>
where
    A: ConsensusApp + Send + Sync + 'static,
    S: EventSink<Block = A::Block> + Send + Sync + 'static,
    A::Block: CommonwareBlock + Send + Sync + 'static,
{
    /// Create a new `CommonwareEngine` with the given app, sink, and config.
    ///
    /// # Arguments
    /// - `app`: The consensus application (implements ConsensusApp)
    /// - `sink`: The event sink for finalization notifications
    /// - `config`: Configuration for the simplex consensus engine
    pub fn new(app: Arc<A>, sink: Arc<S>, config: CommonwareConfig) -> Self {
        Self { app, sink, config }
    }
}

impl<A, S> ConsensusEngine for CommonwareEngine<A, S>
where
    A: ConsensusApp + Send + Sync + 'static,
    S: EventSink<Block = A::Block> + Send + Sync + 'static,
    A::Block: CommonwareBlock + Send + Sync + 'static,
{
    fn start(self) -> Result<RunningEngine, ConsensusError> {
        let height = Arc::new(AtomicU64::new(0));
        let running = Arc::new(AtomicBool::new(false));

        // Create mailbox channel for actor communication
        let (tx, rx) = mpsc::channel(self.config.mailbox_size);
        
        // Create mailbox (implements Automaton/Relay for simplex engine)
        let _mailbox = Mailbox::<A::Block>::new(tx);
        
        // Create mailbox actor (processes messages, delegates to app)
        let _actor = MailboxActor::new(rx, Arc::clone(&height), Arc::clone(&self.app));
        
        // Create finalization sink
        let _finalization_sink = FinalizationSink::<A::Block>::new(Arc::clone(&height));

        // STUB: Simulate block finalization instead of real simplex engine wiring
        // Real implementation would:
        // 1. Create AppAdapter wrapping app + sink
        // 2. Spawn mailbox actor task
        // 3. Configure simplex engine with mailbox as automaton
        // 4. Wire P2P channels, marshal actor, broadcast buffer
        // 5. Start simplex engine and return real shutdown handle

        let running_clone = Arc::clone(&running);
        let height_clone = Arc::clone(&height);

        let handle = std::thread::spawn(move || {
            tracing::info!("Consensus engine thread started (stub mode - simulating finalization)");

            let mut current_height = 0u64;
            let start = std::time::Instant::now();

            // Simple loop checking the running flag and simulating block finalization
            while running_clone.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(100));

                // Simulate block finalization every 5 seconds
                let elapsed_secs = start.elapsed().as_secs();
                let expected_height = elapsed_secs / 5; // 1 block every 5 seconds

                if expected_height > current_height {
                    current_height = expected_height;
                    height_clone.store(current_height, Ordering::SeqCst);
                    tracing::info!("Simulated block finalized at height {}", current_height);
                }
            }

            tracing::info!("Consensus engine thread shutting down");
            Ok(())
        });

        // Wrap thread handle in tokio JoinHandle
        let join_handle: JoinHandle<Result<(), ConsensusError>> =
            tokio::task::spawn_blocking(move || {
                handle
                    .join()
                    .map_err(|e| ConsensusError::Other(format!("Thread panicked: {:?}", e).into()))?
            });

        // Shutdown function
        let running_for_shutdown = Arc::clone(&running);
        let stop_fn = Box::new(move || {
            running_for_shutdown.store(false, Ordering::SeqCst);
            tracing::info!("Shutdown signal sent to consensus engine");
        }) as Box<dyn FnOnce() + Send>;

        running.store(true, Ordering::SeqCst);

        Ok(RunningEngine::new(stop_fn, join_handle, height, running))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{MockApp, TestBlock};
    use crate::sink::FinalizationSink;
    use consensus::engine::ConsensusEngine;
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::time::Duration;

    fn test_config() -> CommonwareConfig {
        CommonwareConfig {
            namespace: "test".to_string(),
            leader_timeout: Duration::from_secs(1),
            notarization_timeout: Duration::from_secs(1),
            nullify_retry: Duration::from_millis(100),
            activity_timeout: 10,
            skip_timeout: 5,
            mailbox_size: 10,
            replay_buffer: NonZeroUsize::new(10).unwrap(),
            write_buffer: NonZeroUsize::new(10).unwrap(),
            epoch: 0,
            fetch_timeout: Duration::from_secs(1),
            fetch_concurrent: 4,
        }
    }

    #[tokio::test]
    async fn test_engine_can_be_constructed() {
        let app = Arc::new(MockApp);
        let height = Arc::new(AtomicU64::new(0));
        let sink = Arc::new(FinalizationSink::<TestBlock>::new(height));
        let config = test_config();

        let _engine = CommonwareEngine::new(app, sink, config);
        // Test passes if construction succeeds
    }

    #[tokio::test]
    async fn test_engine_can_start_and_shutdown() {
        let app = Arc::new(MockApp);
        let height = Arc::new(AtomicU64::new(0));
        let sink = Arc::new(FinalizationSink::<TestBlock>::new(height));
        let config = test_config();

        let engine = CommonwareEngine::new(app, sink, config);
        let running = engine.start().expect("Engine should start");

        // Check status
        let status = running.status();
        assert!(status.is_running);
        assert_eq!(status.current_height, 0);

        // Shutdown
        running.shutdown().await.expect("Shutdown should succeed");
    }

    #[tokio::test]
    async fn test_engine_simulates_block_finalization() {
        let app = Arc::new(MockApp);
        let height = Arc::new(AtomicU64::new(0));
        let sink = Arc::new(FinalizationSink::<TestBlock>::new(Arc::clone(&height)));
        let config = test_config();

        let engine = CommonwareEngine::new(app, sink, config);
        let running = engine.start().expect("Engine should start");

        // Wait for at least 1 simulated block (5 seconds per block in stub)
        tokio::time::sleep(Duration::from_secs(6)).await;

        let current = height.load(Ordering::SeqCst);
        assert!(current >= 1, "Should have finalized at least 1 block");

        // Shutdown
        running.shutdown().await.expect("Shutdown should succeed");
    }
}
