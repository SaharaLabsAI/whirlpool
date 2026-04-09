use crate::error::MempoolError;

/// Trait for mempool storage backends.
pub trait MempoolStore: Send + Sync {
    /// Adds a transaction to the mempool backend.
    fn push(&self, tx: Vec<u8>) -> Result<(), MempoolError>;

    /// Returns all pending transactions and clears them from storage.
    fn drain_pending(&self) -> Result<Vec<Vec<u8>>, MempoolError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn MempoolStore) {}
    }
}
