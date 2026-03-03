use std::future::Future;
use std::sync::Mutex;

pub trait Application: Send + Sync + Clone + 'static {
    type Block: consensus::Block;
    type Result: Clone + Send;
    type Error: std::error::Error + Send + Sync;

    fn genesis(&self) -> impl Future<Output = Self::Block> + Send;

    fn propose(
        &self,
        parent: &Self::Block,
        height: u64,
    ) -> impl Future<Output = Result<(Self::Block, Self::Result), Self::Error>> + Send;

    fn verify(
        &self,
        parent: &Self::Block,
        block: &Self::Block,
    ) -> impl Future<Output = Result<Self::Result, Self::Error>> + Send;
}

pub trait TxSource {
    fn pending(&self) -> Vec<Vec<u8>>;
}

pub struct NoopTxSource;

impl TxSource for NoopTxSource {
    fn pending(&self) -> Vec<Vec<u8>> {
        Vec::new()
    }
}

/// Minimal in-memory transaction pool.
///
/// Stores raw EIP-2718 encoded transaction bytes and serves them
/// via the [`TxSource`] trait. Thread-safe via interior [`Mutex`].
///
/// `pending()` drains the buffer — each transaction is returned at
/// most once.
pub struct InMemoryTxPool {
    txs: Mutex<Vec<Vec<u8>>>,
}

impl InMemoryTxPool {
    /// Create an empty transaction pool.
    pub fn new() -> Self {
        Self {
            txs: Mutex::new(Vec::new()),
        }
    }

    /// Add a raw EIP-2718 encoded transaction to the pool.
    pub fn push(&self, tx: Vec<u8>) {
        self.txs.lock().expect("tx pool lock poisoned").push(tx);
    }
}

impl Default for InMemoryTxPool {
    fn default() -> Self {
        Self::new()
    }
}

impl TxSource for InMemoryTxPool {
    /// Drain and return all pending transactions.
    ///
    /// After this call the pool is empty. Transactions are returned
    /// in FIFO insertion order.
    fn pending(&self) -> Vec<Vec<u8>> {
        std::mem::take(&mut *self.txs.lock().expect("tx pool lock poisoned"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_tx_source_returns_empty() {
        let source = NoopTxSource;
        assert!(source.pending().is_empty());
    }

    #[test]
    fn new_pool_is_empty() {
        let pool = InMemoryTxPool::new();
        assert!(pool.pending().is_empty());
    }

    #[test]
    fn push_single_tx() {
        let pool = InMemoryTxPool::new();
        pool.push(vec![1, 2, 3]);
        let txs = pool.pending();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0], vec![1, 2, 3]);
    }

    #[test]
    fn push_multiple_txs_fifo_order() {
        let pool = InMemoryTxPool::new();
        pool.push(vec![1]);
        pool.push(vec![2]);
        pool.push(vec![3]);
        let txs = pool.pending();
        assert_eq!(txs, vec![vec![1], vec![2], vec![3]]);
    }

    #[test]
    fn pending_drains_buffer() {
        let pool = InMemoryTxPool::new();
        pool.push(vec![1]);
        let first = pool.pending();
        assert_eq!(first.len(), 1);
        let second = pool.pending();
        assert!(second.is_empty());
    }

    #[test]
    fn push_after_drain() {
        let pool = InMemoryTxPool::new();
        pool.push(vec![1]);
        let _ = pool.pending(); // drain
        pool.push(vec![2]);
        let txs = pool.pending();
        assert_eq!(txs, vec![vec![2]]);
    }

    #[test]
    fn concurrent_push() {
        use std::sync::Arc;
        use std::thread;

        let pool = Arc::new(InMemoryTxPool::new());
        let n = 100;
        let mut handles = Vec::new();

        for i in 0..n {
            let pool = Arc::clone(&pool);
            handles.push(thread::spawn(move || {
                pool.push(vec![i as u8]);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let txs = pool.pending();
        assert_eq!(txs.len(), n);
    }
}
