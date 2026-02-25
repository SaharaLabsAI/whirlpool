//! Wiring for whirlpool-node: starter closure that launches the consensus stack.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use consensus::error::ConsensusError;

/// Creates the starter closure for `CommonwareEngine`.
///
/// STUB IMPLEMENTATION: Simulates block finalization by incrementing height every 5 seconds.
/// Full implementation (simplex engine + P2P + mailbox actor) is future work.
pub fn create_starter() -> impl FnOnce(
    Arc<AtomicU64>,
    Arc<AtomicBool>,
) -> Result<
    (
        Box<dyn FnOnce() + Send>,
        tokio::task::JoinHandle<Result<(), ConsensusError>>,
    ),
    ConsensusError,
> + Send
       + 'static {
    move |height: Arc<AtomicU64>, running: Arc<AtomicBool>| {
        // Stub implementation: simulate block finalization every 5 seconds
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
        let join_handle = tokio::task::spawn_blocking(move || {
            handle
                .join()
                .map_err(|e| ConsensusError::Other(format!("Thread panicked: {:?}", e).into()))?
        });

        // Shutdown function
        let stop_fn = Box::new(move || {
            running.store(false, Ordering::SeqCst);
            tracing::info!("Shutdown signal sent to consensus engine");
        }) as Box<dyn FnOnce() + Send>;

        Ok((stop_fn, join_handle))
    }
}
