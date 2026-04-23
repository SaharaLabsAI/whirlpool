use reth_db::DatabaseError;

/// Errors specific to the reth-backed state database.
#[derive(Debug, thiserror::Error)]
pub enum RethStateError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("initialization error: {0}")]
    Init(String),

    #[error("codec error: {0}")]
    Codec(String),

    #[error("state root error: {0}")]
    StateRoot(String),
}

impl revm::database::DBErrorMarker for RethStateError {}
