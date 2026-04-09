use std::path::Path;

use app::traits::TxSource;
use mempool::MempoolError;

use crate::MdbxMempoolStore;

pub struct PersistentTxPool {
    store: MdbxMempoolStore,
}

impl PersistentTxPool {
    pub fn open(path: &Path) -> Result<Self, MempoolError> {
        let store = MdbxMempoolStore::open(path)?;
        Ok(Self { store })
    }
}

impl TxSource for PersistentTxPool {
    fn push(&self, tx: Vec<u8>) {
        if let Err(err) = self.store.push(tx) {
            eprintln!("persistent tx pool push failed: {err}");
        }
    }

    fn pending(&self) -> Vec<Vec<u8>> {
        match self.store.drain_pending() {
            Ok(txs) => txs,
            Err(err) => {
                eprintln!("persistent tx pool pending failed: {err}");
                Vec::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_txsource_trait_object() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("mdbx");

        let pool = Arc::new(PersistentTxPool::open(&db_path).unwrap());
        let source: Arc<dyn TxSource> = pool;

        source.push(vec![1, 2, 3]);
        assert_eq!(source.pending(), vec![vec![1, 2, 3]]);
    }

    #[test]
    fn test_pending_drains() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("mdbx");

        let pool = PersistentTxPool::open(&db_path).unwrap();
        pool.push(vec![9]);

        assert_eq!(pool.pending(), vec![vec![9]]);
        assert!(pool.pending().is_empty());
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("mdbx");

        {
            let pool = PersistentTxPool::open(&db_path).unwrap();
            pool.push(vec![4]);
            pool.push(vec![5]);
        }

        let reopened = PersistentTxPool::open(&db_path).unwrap();
        assert_eq!(reopened.pending(), vec![vec![4], vec![5]]);
    }
}
