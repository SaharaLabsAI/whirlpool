# mempool

## Purpose

[PROPOSED] Provides persistent transaction pool storage via embedded MDBX database, implementing `TxSource` trait with crash-recoverable semantics. Preserves FIFO ordering and drain-on-pending behavior while surviving node restarts. Encapsulates all database operations, keeping app crate DB-agnostic.

## Public API

[PROPOSED] Core types and implementations:

```rust
pub struct PersistentTxPool { /* private: Arc<Mutex<MempoolStore>> */ }

impl PersistentTxPool {
    /// Open or create persistent mempool at specified path.
    /// Recovers unproposed transactions from previous run.
    /// Fatal error if database cannot be initialized.
    pub fn open(path: PathBuf) -> Result<Self, MempoolError>;
}

impl TxSource for PersistentTxPool {
    /// Drain all pending transactions in FIFO insertion order.
    /// Atomically reads and deletes all txs from database.
    /// After this call, database is empty until new pushes.
    fn pending(&self) -> Vec<Vec<u8>>;
    
    /// Submit raw EIP-2718 encoded transaction to persistent pool.
    /// Assigns auto-increment u64 key for FIFO ordering.
    /// Persists immediately, survives crashes.
    fn push(&self, tx: Vec<u8>);
}

#[derive(Debug)]
pub enum MempoolError {
    DatabaseOpen(String),      // MDBX initialization failure
    WriteTransaction(String),   // Persist operation failure
    ReadTransaction(String),    // Drain operation failure
}
```

[PROPOSED] Internal implementation (not public):

```rust
struct MempoolStore {
    env: Environment,           // MDBX environment handle
    db: Database,               // MDBX database handle
    next_id: AtomicU64,         // Auto-increment counter for keys
}

impl MempoolStore {
    fn push(&mut self, tx: Vec<u8>) -> Result<(), MempoolError>;
    fn drain_pending(&mut self) -> Result<Vec<Vec<u8>>, MempoolError>;
}
```

## Config

[PROPOSED] Configuration via `PersistentTxPool::open()` parameters:

- **path**: `PathBuf` — database directory (e.g., `{persistent_storage_dir}/mempool`)
- **MDBX parameters** (internal defaults):
  - Max DB size: 1 GB initial (MDBX grows dynamically)
  - Max readers: 126 (MDBX default)
  - No encryption (embedded database, rely on filesystem security)

[PROPOSED] No runtime configuration — behavior fixed to match `InMemoryTxPool` semantics.

## Internal Modules

[PROPOSED] Module structure:

- `lib.rs` — re-exports `PersistentTxPool`, `MempoolError`
- `persistent.rs` — `PersistentTxPool` struct and `TxSource` trait implementation
- `store.rs` — `MempoolStore` wrapper for raw MDBX operations

## Primary Flows

### Flow 1: Initialization [PROPOSED]
```
PersistentTxPool::open(path)
  → MempoolStore::open(path)
    → libmdbx: Environment::new(path)
    → libmdbx: env.create_db(None) // unnamed database
    → Scan database for max key, resume counter
    → Return PersistentTxPool wrapping store in Arc<Mutex<>>
```

**Crash Recovery**: On startup, any txs remaining in database are unproposed → returned by next `pending()` call.

### Flow 2: Transaction Submission [PROPOSED]
```
TxSource::push(tx: Vec<u8>)
  → Acquire Mutex<MempoolStore>
  → next_id.fetch_add(1)
  → MDBX write transaction:
      put(next_id.to_be_bytes(), tx)
      commit()
  → Release mutex
```

**Error Handling**: On MDBX write failure, log error and silently drop tx (preserves trait infallibility). Client must retry submission.

**Concurrency**: Mutex ensures serialized writes (MDBX single-writer constraint). Multiple concurrent `push()` calls queue on mutex.

### Flow 3: Transaction Drain [PROPOSED]
```
TxSource::pending() -> Vec<Vec<u8>>
  → Acquire Mutex<MempoolStore>
  → MDBX write transaction:
      cursor.iter() // iterate all keys ascending
      collect values into Vec
      cursor.del() for each key
      commit()
  → Release mutex
  → Return Vec (FIFO order preserved)
```

**Atomicity**: Read + delete in single MDBX transaction — crash during drain leaves txs in database (recovered on restart).

**Error Handling**: On MDBX read/delete failure, log error and return empty vec (preserves trait infallibility, consensus proposes empty block).

### Flow 4: Restart Recovery [PROPOSED]
```
Node crash → restart → PersistentTxPool::open(path)
  → Database already exists, open existing environment
  → Scan for max key to resume counter
  → Next pending() call returns all persisted txs
```

**Semantics**: Txs persisted but not yet proposed are recovered. Txs drained by `pending()` before crash are lost (matches `InMemoryTxPool` behavior).

## Dependencies

[PROPOSED] Crate dependencies:

- `app` — for `TxSource` trait definition
- `libmdbx-rs` — raw MDBX bindings (NOT reth-db, see rationale below)
- `parking_lot` — lightweight `Mutex` for `MempoolStore` interior mutability
- `tracing` — for error logging

**Rationale for raw libmdbx-rs**:
- [GROUNDED] reth-db's `Tables` enum is vendor-controlled, cannot be extended without modifying vendor code
- [PROPOSED] MDBX supports multiple independent databases via separate paths — mempool DB coexists with state DB
- [PROPOSED] Mempool schema is trivial (u64 → bytes) — reth-db abstractions (`Table` trait, codec) unnecessary
- [GROUNDED] state-reth already uses reth-db for state persistence — demonstrates MDBX coexistence pattern

## Error Types

[PROPOSED] Error handling strategy:

- **Startup errors** (fatal): `PersistentTxPool::open()` returns `Result<Self, MempoolError>`. Node wiring propagates to `main()`, logs diagnostic, exits process.
- **Runtime errors** (non-fatal): `push()` and `pending()` log errors internally, degrade gracefully (drop tx or return empty vec). Trait contract remains infallible.

```rust
#[derive(Debug)]
pub enum MempoolError {
    DatabaseOpen(String),      // Path invalid, permissions denied, disk full
    WriteTransaction(String),   // Persist failure (disk I/O, corruption)
    ReadTransaction(String),    // Drain failure (corruption, concurrent write from external process)
}

impl std::fmt::Display for MempoolError { /* ... */ }
impl std::error::Error for MempoolError {}
```

## Invariants and Contracts

[PROPOSED] Trait contract guarantees:

1. **FIFO ordering**: `pending()` returns transactions in insertion order (ascending key order).
2. **Drain semantics**: `pending()` atomically drains database — subsequent calls return empty vec until new pushes.
3. **Thread safety**: Multiple concurrent `push()` calls safe via `Mutex`. Single `pending()` call safe (called only by consensus thread).
4. **Crash recovery**: Transactions persisted before crash but not yet drained are recovered on restart.
5. **Infallibility**: `push()` and `pending()` never panic or return errors to caller (errors logged internally).

[PROPOSED] Internal invariants:

1. **Key monotonicity**: `next_id` counter always increments, never reused (even after drain).
2. **Atomic drain**: Read-all + delete-all in single MDBX write transaction — no partial drains visible.
3. **Counter recovery**: On `open()`, counter initialized to `max(existing_keys) + 1` or 0 if empty.

[PROPOSED] Non-guarantees (intentional, match `InMemoryTxPool`):

- **No deduplication**: Same transaction submitted twice results in two entries with different keys.
- **No validation**: Raw bytes stored without decoding — invalid EIP-2718 txs accepted (validation deferred to `EvmApplication::propose()`).
- **No finalization tracking**: Crash between `pending()` drain and consensus finalization loses txs (same as `InMemoryTxPool`).

## Storage Schema

[PROPOSED] MDBX database schema:

```
Database: unnamed (default)
Table: pending_txs (implicit via MDBX key-value pairs)
  Key: u64 (8 bytes, big-endian) — auto-increment insertion order
  Value: Vec<u8> (variable length) — raw EIP-2718 encoded transaction

Metadata: none (counter recovered from max key)
```

**Key encoding**: `u64::to_be_bytes()` ensures lexicographic order matches numeric order (ascending iteration yields FIFO).

**Value encoding**: No encoding — raw bytes stored directly.

## Open Questions

[PROPOSED] Future enhancements (out of scope for MVP):

1. **Deduplication**: Add secondary index on tx hash for O(1) duplicate detection.
2. **Lifecycle tracking**: Add `proposed_txs` table to track submitted → proposed → finalized states, enable re-queuing on crash.
3. **Metrics**: Expose `mempool_push_errors_total`, `mempool_drain_errors_total`, `mempool_size_bytes` for observability.
4. **Compaction**: Periodically compact MDBX database to reclaim space from deleted keys (counter grows unbounded, keys never reused).
5. **Benchmarking**: Measure actual overhead vs `InMemoryTxPool` on target hardware (expect <1ms per op).

[PROPOSED] Design decisions requiring validation:

- **MDBX vs alternatives**: Assume MDBX is best fit. If performance issues emerge, consider redb (pure Rust, simpler API) or sled (beta but async-friendly).
- **Coarse-grained locking**: Single `Mutex` for entire store. If contention observed, consider finer-grained locking (separate read/write locks, per-operation transactions).
