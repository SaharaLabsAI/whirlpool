use app::ApplicationError;

#[derive(Debug, thiserror::Error)]
pub enum EvmAppError {
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("State root mismatch: expected {expected:?}, computed {computed:?}")]
    StateRootMismatch {
        expected: [u8; 32],
        computed: [u8; 32],
    },
    #[error("State error: {0}")]
    State(String),
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
}

impl From<core::convert::Infallible> for EvmAppError {
    fn from(e: core::convert::Infallible) -> Self {
        match e {}
    }
}

impl From<state::StateError> for EvmAppError {
    fn from(err: state::StateError) -> Self {
        EvmAppError::State(err.to_string())
    }
}

impl From<state_reth::RethStateError> for EvmAppError {
    fn from(err: state_reth::RethStateError) -> Self {
        EvmAppError::State(err.to_string())
    }
}

impl From<EvmAppError> for ApplicationError {
    fn from(err: EvmAppError) -> Self {
        match err {
            EvmAppError::Execution(message) => ApplicationError::Execution(message),
            EvmAppError::StateRootMismatch { expected, computed } => {
                ApplicationError::Verification(format!(
                    "State root mismatch: expected {:?}, computed {:?}",
                    expected, computed
                ))
            }
            EvmAppError::State(message) => ApplicationError::State(message),
            EvmAppError::InvalidBlock(message) => ApplicationError::Verification(message),
            EvmAppError::InvalidTransaction(message) => ApplicationError::Verification(message),
        }
    }
}
