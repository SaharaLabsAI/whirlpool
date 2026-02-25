//! CommonwareEngine — bridges commonware simplex BFT to the `ConsensusEngine` trait.
//!
//! Because the simplex stack requires extensive infrastructure (P2P channels,
//! storage backends, buffer pools, marshal actor, etc.), this module provides
//! a lightweight wrapper that accepts a pre-built start routine and implements
//! the `ConsensusEngine` trait from consensus-core.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use consensus::engine::{ConsensusEngine, RunningEngine};
use consensus::error::ConsensusError;

/// A consensus engine backed by the Commonware Simplex BFT protocol.
///
/// `CommonwareEngine` wraps a start closure that, when invoked, launches the
/// full simplex consensus stack (marshal, broadcast buffer, simplex engine)
/// and returns a shutdown handle plus a task join handle.
///
/// # Construction
///
/// Use [`CommonwareEngine::new`] with a closure that performs the actual engine
/// startup. The closure receives shared `height` and `running` atomics for
/// status reporting and must return a boxed shutdown function plus a tokio
/// `JoinHandle`.
///
/// # Example (sketch)
///
/// ```ignore
/// let engine = CommonwareEngine::new(|height, running| {
///     // Build marshal, Marshaled, simplex::Engine, start them...
///     // Update `height` and `running` atomics as consensus progresses.
///     let shutdown = Box::new(|| { /* signal shutdown */ }) as Box<dyn FnOnce() + Send>;
///     let handle = tokio::spawn(async move { /* run loop */ Ok(()) });
///     Ok((shutdown, handle))
/// });
/// let running = engine.start()?;
/// ```
pub struct CommonwareEngine {
    /// The start routine that launches the consensus stack.
    /// Receives (height_atomic, running_atomic) for status reporting.
    /// Returns (shutdown_fn, join_handle) or an error.
    starter: Box<
        dyn FnOnce(
                Arc<AtomicU64>,
                Arc<AtomicBool>,
            )
                -> Result<
                    (
                        Box<dyn FnOnce() + Send>,
                        tokio::task::JoinHandle<Result<(), ConsensusError>>,
                    ),
                    ConsensusError,
                > + Send,
    >,
}

impl CommonwareEngine {
    /// Create a new `CommonwareEngine` with the given start routine.
    ///
    /// The `starter` closure is called exactly once when [`ConsensusEngine::start`]
    /// is invoked. It receives:
    /// - `height`: An `Arc<AtomicU64>` that the engine should update as blocks finalize.
    /// - `running`: An `Arc<AtomicBool>` that the engine should set to `true` while
    ///   running and `false` on shutdown.
    ///
    /// It must return:
    /// - A shutdown function that signals the engine to stop.
    /// - A `JoinHandle` that resolves when the engine terminates.
    pub fn new<F>(starter: F) -> Self
    where
        F: FnOnce(
                Arc<AtomicU64>,
                Arc<AtomicBool>,
            )
                -> Result<
                    (
                        Box<dyn FnOnce() + Send>,
                        tokio::task::JoinHandle<Result<(), ConsensusError>>,
                    ),
                    ConsensusError,
                > + Send
            + 'static,
    {
        Self {
            starter: Box::new(starter),
        }
    }
}

impl ConsensusEngine for CommonwareEngine {
    fn start(self) -> Result<RunningEngine, ConsensusError> {
        let height = Arc::new(AtomicU64::new(0));
        let running = Arc::new(AtomicBool::new(false));

        let (shutdown, handle) =
            (self.starter)(Arc::clone(&height), Arc::clone(&running))?;

        running.store(true, Ordering::SeqCst);

        Ok(RunningEngine::new(shutdown, handle, height, running))
    }
}
