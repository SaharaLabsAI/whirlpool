use app::ApplicationError;
use app_evm::EvmAppError;

#[derive(Debug, thiserror::Error)]
pub enum CompositeAppError {
    #[error(transparent)]
    Evm(#[from] EvmAppError),
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),
}

impl From<CompositeAppError> for ApplicationError {
    fn from(err: CompositeAppError) -> Self {
        match err {
            CompositeAppError::Evm(err) => err.into(),
            CompositeAppError::InvalidTransaction(message) => {
                ApplicationError::Verification(message)
            }
        }
    }
}
