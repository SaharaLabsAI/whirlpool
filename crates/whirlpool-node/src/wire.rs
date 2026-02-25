//! Wiring for whirlpool-node: starter closure that launches the consensus stack.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use consensus::error::ConsensusError;

/// Creates the starter closure for `CommonwareEngine`.
///
/// MINIMAL IMPLEMENTATION: Returns a background thread that checks the running flag.
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
    move |_height: Arc<AtomicU64>, running: Arc<AtomicBool>| {
        // Minimal implementation: background thread checking running flag
        let running_clone = Arc::clone(&running);

        let handle = std::thread::spawn(move || {
            tracing::info!("Consensus engine thread started");

            // Simple loop checking the running flag
            while running_clone.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(100));
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
