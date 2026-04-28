use app_primitives::{EvmBlock, Receipt};
use revm::primitives::B256;

/// Errors that can occur during block storage operations.
#[derive(Debug, thiserror::Error)]
pub enum BlockStorageError {
    #[error("database error: {0}")]
    Database(String),

    #[error("codec error: {0}")]
    Codec(String),
}

/// Persistent storage for finalized blocks and their receipts.
///
/// Implementations must be thread-safe and provide atomic writes —
/// a `store_block` call either fully persists the block and its receipts
/// or fails without partial writes.
pub trait BlockStorage: Send + Sync {
    /// Atomically persist a finalized block and its associated receipts.
    ///
    /// Implementations must store:
    /// - Block header fields (indexed by both number and hash)
    /// - Transaction bodies
    /// - Receipts (indexed by block number)
    ///
    /// Calling this with an already-stored block number should be idempotent.
    fn store_block(&self, block: &EvmBlock, receipts: &[Receipt]) -> Result<(), BlockStorageError>;

    /// Retrieve a block by its height/number.
    ///
    /// Returns `None` if no block is stored at the given height.
    fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>, BlockStorageError>;

    /// Retrieve a block by its hash.
    ///
    /// Returns `None` if no block with the given hash is stored.
    fn get_block_by_hash(&self, hash: B256) -> Result<Option<EvmBlock>, BlockStorageError>;

    /// Retrieve receipts for a block by its height/number.
    ///
    /// Returns `None` if no block is stored at the given height.
    fn get_receipts_by_block(&self, number: u64)
        -> Result<Option<Vec<Receipt>>, BlockStorageError>;

    /// Return the highest block number stored, or `None` if the store is empty.
    ///
    /// Used on startup to determine whether to resume from an existing chain
    /// or initialise from genesis.
    fn get_latest_block_number(&self) -> Result<Option<u64>, BlockStorageError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-ST-01: Verify BlockStorage is object-safe and satisfies Send + Sync bounds.
    #[test]
    fn block_storage_is_object_safe_send_sync() {
        fn _assert_object_safe(_: &dyn BlockStorage) {}
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        // If this compiles, the trait is object-safe and Send + Sync.
        assert_send::<Box<dyn BlockStorage>>();
        assert_sync::<Box<dyn BlockStorage>>();
    }
}
