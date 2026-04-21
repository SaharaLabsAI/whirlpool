use crate::engine::RunningEngine;
use crate::error::ConsensusError;

impl RunningEngine {
    /// Signal the engine to shut down, then wait for it to terminate.
    pub async fn shutdown(self) -> Result<(), ConsensusError> {
        (self._shutdown)();
        self.handle
            .await
            .map_err(|e| ConsensusError::Runtime(e.to_string()))?
    }
}
