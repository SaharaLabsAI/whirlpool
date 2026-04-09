//! Cross-crate integration tests for the mempool crate.
//!
//! Validates that `PersistentTxPool` satisfies the `TxSource` trait contract
//! across crate boundaries, including crash-recovery, FIFO ordering, and
//! trait-object coercion.

use std::sync::Arc;

use app::traits::TxSource;
use mempool_mdbx::PersistentTxPool;
use tempfile::TempDir;

/// INT-FLOW-01: PersistentTxPool can be used as `Arc<dyn TxSource>`
/// across crate boundaries (the same way EthRpcContext and EvmApplication
/// consume the pool).
#[test]
fn trait_object_coercion_across_crates() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("mdbx");

    let pool = Arc::new(PersistentTxPool::open(&db_path).unwrap());
    let source: Arc<dyn TxSource> = pool;

    source.push(vec![0xAA, 0xBB]);
    source.push(vec![0xCC]);

    let txs = source.pending();
    assert_eq!(txs.len(), 2);
    assert_eq!(txs[0], vec![0xAA, 0xBB]);
    assert_eq!(txs[1], vec![0xCC]);
}

/// INT-CR-01: Transactions survive a close-and-reopen cycle, proving
/// on-disk persistence through the `TxSource` trait interface.
#[test]
fn restart_recovery_via_trait() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("mdbx");

    // Phase 1: push transactions through trait object, then drop.
    {
        let pool: Arc<dyn TxSource> = Arc::new(PersistentTxPool::open(&db_path).unwrap());
        pool.push(vec![1, 2, 3]);
        pool.push(vec![4, 5]);
    }

    // Phase 2: reopen and verify transactions survive.
    {
        let pool: Arc<dyn TxSource> = Arc::new(PersistentTxPool::open(&db_path).unwrap());
        let txs = pool.pending();
        assert_eq!(txs, vec![vec![1, 2, 3], vec![4, 5]]);
    }
}

/// INT-CR-02: After draining and re-opening, pool is empty — drain
/// is durable (deletes are committed to disk).
#[test]
fn restart_after_drain_is_empty() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("mdbx");

    // Push and drain.
    {
        let pool = PersistentTxPool::open(&db_path).unwrap();
        pool.push(vec![10]);
        let txs = pool.pending(); // drains
        assert_eq!(txs, vec![vec![10]]);
    }

    // Reopen — should be empty.
    {
        let pool = PersistentTxPool::open(&db_path).unwrap();
        let txs = pool.pending();
        assert!(txs.is_empty(), "pool should be empty after drain + restart");
    }
}

/// INT-FLOW-04: FIFO ordering is maintained end-to-end through the
/// TxSource trait interface across push/pending cycles.
#[test]
fn fifo_ordering_preserved() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("mdbx");

    let pool: Arc<dyn TxSource> = Arc::new(PersistentTxPool::open(&db_path).unwrap());

    // Push in a specific order.
    for i in 0u8..10 {
        pool.push(vec![i]);
    }

    // Verify FIFO.
    let txs = pool.pending();
    assert_eq!(txs.len(), 10);
    for (i, tx) in txs.iter().enumerate() {
        assert_eq!(tx, &vec![i as u8], "tx at index {i} should be [{i}]");
    }
}

/// INT-FLOW-04 (extended): FIFO ordering survives a restart.
#[test]
fn fifo_ordering_survives_restart() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("mdbx");

    {
        let pool = PersistentTxPool::open(&db_path).unwrap();
        pool.push(vec![0xA0]);
        pool.push(vec![0xB0]);
        pool.push(vec![0xC0]);
    }

    let pool = PersistentTxPool::open(&db_path).unwrap();
    let txs = pool.pending();
    assert_eq!(
        txs,
        vec![vec![0xA0], vec![0xB0], vec![0xC0]],
        "FIFO order must be preserved across restart"
    );
}

/// Multi-threaded push through trait object (validates Send + Sync bounds).
#[test]
fn concurrent_push_via_trait_object() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("mdbx");

    let pool: Arc<dyn TxSource> = Arc::new(PersistentTxPool::open(&db_path).unwrap());

    let handles: Vec<_> = (0u8..4)
        .map(|i| {
            let pool = Arc::clone(&pool);
            std::thread::spawn(move || {
                pool.push(vec![i]);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let txs = pool.pending();
    assert_eq!(txs.len(), 4, "all concurrent pushes should be recorded");
}
