use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use mempool::MempoolError;
use reth_libmdbx::{Database, Environment, WriteFlags};

pub struct MdbxMempoolStore {
    env: Environment,
    db: Database,
    next_key: AtomicU64,
}

impl MdbxMempoolStore {
    pub fn open(path: &Path) -> Result<Self, MempoolError> {
        fs::create_dir_all(path)?;

        let env = Environment::builder().open(path)?;

        let db = {
            let tx = env.begin_rw_txn()?;
            let db = tx.open_db(None)?;
            tx.commit()?;
            db
        };

        let next_key = Self::load_next_key(&env, &db)?;

        Ok(Self { env, db, next_key: AtomicU64::new(next_key) })
    }

    pub fn push(&self, tx: Vec<u8>) -> Result<(), MempoolError> {
        let key = self.next_key.fetch_add(1, Ordering::Relaxed).to_be_bytes();

        let rw_tx = self.env.begin_rw_txn()?;
        rw_tx.put(self.db.dbi(), key, tx, WriteFlags::empty())?;
        rw_tx.commit()?;

        Ok(())
    }

    pub fn drain_pending(&self) -> Result<Vec<Vec<u8>>, MempoolError> {
        let rw_tx = self.env.begin_rw_txn()?;
        let mut cursor = rw_tx.cursor(self.db.dbi())?;

        let mut txs = Vec::new();
        for item in cursor.iter_start::<[u8; 8], Vec<u8>>() {
            let (_key, value) = item?;
            txs.push(value);
        }

        rw_tx.clear_db(self.db.dbi())?;
        rw_tx.commit()?;

        Ok(txs)
    }

    fn load_next_key(env: &Environment, db: &Database) -> Result<u64, MempoolError> {
        let ro_tx = env.begin_ro_txn()?;
        let mut cursor = ro_tx.cursor(db.dbi())?;

        let next = match cursor.last::<[u8; 8], ()>()? {
            Some((key, _)) => u64::from_be_bytes(key).saturating_add(1),
            None => 0,
        };

        Ok(next)
    }
}

impl mempool::MempoolStoreTrait for MdbxMempoolStore {
    fn push(&self, tx: Vec<u8>) -> Result<(), MempoolError> {
        MdbxMempoolStore::push(self, tx)
    }

    fn drain_pending(&self) -> Result<Vec<Vec<u8>>, MempoolError> {
        MdbxMempoolStore::drain_pending(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeSet, sync::Arc, thread};
    use tempfile::TempDir;

    #[test]
    fn implements_mempool_store_trait() {
        fn _assert_impl<T: mempool::MempoolStoreTrait>() {}
        _assert_impl::<MdbxMempoolStore>();
    }

    fn new_store(temp_dir: &TempDir) -> MdbxMempoolStore {
        let db_path = temp_dir.path().join("mdbx");
        MdbxMempoolStore::open(&db_path).expect("open store")
    }

    #[test]
    fn test_push_and_drain() {
        let temp_dir = TempDir::new().unwrap();
        let store = new_store(&temp_dir);

        store.push(vec![1]).unwrap();
        store.push(vec![2]).unwrap();
        store.push(vec![3]).unwrap();

        let drained = store.drain_pending().unwrap();
        assert_eq!(drained, vec![vec![1], vec![2], vec![3]]);
    }

    #[test]
    fn test_drain_empty() {
        let temp_dir = TempDir::new().unwrap();
        let store = new_store(&temp_dir);

        assert!(store.drain_pending().unwrap().is_empty());
    }

    #[test]
    fn test_drain_clears() {
        let temp_dir = TempDir::new().unwrap();
        let store = new_store(&temp_dir);

        store.push(vec![9]).unwrap();
        let first = store.drain_pending().unwrap();
        assert_eq!(first, vec![vec![9]]);

        let second = store.drain_pending().unwrap();
        assert!(second.is_empty());
    }

    #[test]
    fn test_persistence_across_reopen() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("mdbx");

        {
            let store = MdbxMempoolStore::open(&db_path).unwrap();
            store.push(vec![7, 7]).unwrap();
            store.push(vec![8, 8]).unwrap();
        }

        let reopened = MdbxMempoolStore::open(&db_path).unwrap();
        let drained = reopened.drain_pending().unwrap();
        assert_eq!(drained, vec![vec![7, 7], vec![8, 8]]);
    }

    #[test]
    fn test_fifo_ordering() {
        let temp_dir = TempDir::new().unwrap();
        let store = new_store(&temp_dir);

        let inputs = vec![
            b"first".to_vec(),
            b"second".to_vec(),
            b"third".to_vec(),
            b"fourth".to_vec(),
        ];

        for tx in &inputs {
            store.push(tx.clone()).unwrap();
        }

        let drained = store.drain_pending().unwrap();
        assert_eq!(drained, inputs);
    }

    #[test]
    fn test_multiple_push_drain_cycles() {
        let temp_dir = TempDir::new().unwrap();
        let store = new_store(&temp_dir);

        store.push(vec![1]).unwrap();
        store.push(vec![2]).unwrap();
        assert_eq!(store.drain_pending().unwrap(), vec![vec![1], vec![2]]);

        store.push(vec![3]).unwrap();
        store.push(vec![4]).unwrap();
        store.push(vec![5]).unwrap();
        assert_eq!(store.drain_pending().unwrap(), vec![vec![3], vec![4], vec![5]]);
    }

    #[test]
    fn test_concurrent_push() {
        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(new_store(&temp_dir));
        let threads = 8usize;
        let per_thread = 25usize;

        let mut handles = Vec::new();
        for t in 0..threads {
            let store = Arc::clone(&store);
            handles.push(thread::spawn(move || {
                for i in 0..per_thread {
                    let value = format!("tx-{}-{}", t, i).into_bytes();
                    store.push(value).unwrap();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let drained = store.drain_pending().unwrap();
        assert_eq!(drained.len(), threads * per_thread);

        let actual: BTreeSet<Vec<u8>> = drained.into_iter().collect();
        let expected: BTreeSet<Vec<u8>> = (0..threads)
            .flat_map(|t| (0..per_thread).map(move |i| format!("tx-{}-{}", t, i).into_bytes()))
            .collect();

        assert_eq!(actual, expected);
    }
}
