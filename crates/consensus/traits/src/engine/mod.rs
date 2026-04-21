use crate::error::ConsensusError;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;

mod shutdown;

/// Snapshot of the consensus engine's current status.
#[derive(Debug, Clone, Copy)]
pub struct ConsensusStatus {
    pub current_height: u64,
    pub is_running: bool,
}

/// A running consensus engine handle.
///
/// Created by [`ConsensusEngine::start`]. Provides status queries
/// and graceful shutdown.
pub struct RunningEngine {
    _shutdown: Box<dyn FnOnce() + Send>,
    handle: JoinHandle<Result<(), ConsensusError>>,
    height: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
}

impl RunningEngine {
    /// Create a new RunningEngine from its components.
    pub fn new(
        shutdown: Box<dyn FnOnce() + Send>,
        handle: JoinHandle<Result<(), ConsensusError>>,
        height: Arc<AtomicU64>,
        running: Arc<AtomicBool>,
    ) -> Self {
        Self {
            _shutdown: shutdown,
            handle,
            height,
            running,
        }
    }

    /// Query the current consensus status.
    pub fn status(&self) -> ConsensusStatus {
        ConsensusStatus {
            current_height: self.height.load(Ordering::Relaxed),
            is_running: self.running.load(Ordering::Relaxed),
        }
    }

    /// Wait for the engine to terminate, returning its result.
    pub async fn wait(self) -> Result<(), ConsensusError> {
        self.handle
            .await
            .map_err(|e| ConsensusError::Runtime(e.to_string()))?
    }
}

/// Trait for types that can start a consensus engine.
pub trait ConsensusEngine {
    /// Start the consensus engine, returning a handle to the running instance.
    fn start(self) -> Result<RunningEngine, ConsensusError>;
}
