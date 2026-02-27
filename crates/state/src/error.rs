#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Internal error: {0}")]
    Internal(String),
}

// Implement DBErrorMarker so StateError can be used with revm Database trait
impl revm::database::DBErrorMarker for StateError {}
