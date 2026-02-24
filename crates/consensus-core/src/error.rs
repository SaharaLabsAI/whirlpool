#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("invalid block: {0}")]
    InvalidBlock(String),

    #[error("proposal failed: {0}")]
    ProposalFailed(String),

    #[error("not ready: {0}")]
    NotReady(String),

    #[error("runtime error: {0}")]
    Runtime(String),

    #[error("consensus engine shut down")]
    Shutdown,

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
