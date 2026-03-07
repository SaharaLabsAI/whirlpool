# Domains & Wiring

## Overview

This document defines the domain boundaries, module responsibilities, and cross-domain wiring for the persistent mempool implementation. The design introduces a new persistence domain while preserving existing trait contracts and consensus semantics.

**Status Legend**: `[GROUNDED]` = based on existing code, `[PROPOSED]` = new design decisions, `UNKNOWN` = requires investigation, `BLOCKER` = contract-critical gap.

---

## Domains

### 1. Trait Domain (app crate)

**Bounded Context**: Application-layer trait definitions and shared types. Defines the contract for transaction sources without specifying implementation details.

**Owning Crate**: `app` (`crates/app/`)

**Key Types & Traits**:
- `TxSource` trait [GROUNDED: `app/src/traits.rs:23`]
  - [GROUNDED] Current: `fn pending(&self) -> Vec<Vec<u8>>`
  - [PROPOSED] Extended: `fn push(&self, tx: Vec<u8>)`
- `InMemoryTxPool` [GROUNDED: `app/src/tx_source.rs:30-45`]
  - Implementation: `Mutex<Vec<Vec<u8>>>` with FIFO drain semantics
- `NoopTxSource` [GROUNDED: `app/src/tx_source.rs:47-55`]
  - Test/stub implementation returning empty vectors

**Responsibilities**:
- Define `TxSource` trait contract (push + drain operations)
- Provide reference implementation (`InMemoryTxPool`) for testing and fallback
- Ensure trait bounds support concurrent access (`Send + Sync`)

**Dependencies**: None (foundational crate)

**Public API**:
```rust
pub trait TxSource: Send + Sync {
    fn pending(&self) -> Vec<Vec<u8>>;      // [GROUNDED]
    fn push(&self, tx: Vec<u8>);             // [PROPOSED]
}
```

**Error Types**: None exposed by trait (implementors handle internally)

**Evidence**:
- `app/src/traits.rs:23` — current trait definition
- `app/src/tx_source.rs:36` — `InMemoryTxPool::pending()` drain via `std::mem::take`
- `app/src/tx_source.rs:30-45` — concurrent safety via `Mutex`

---

### 2. Persistence Domain (mempool crate)

**Bounded Context**: Persistent transaction storage using embedded MDBX database. Implements `TxSource` trait with crash-recoverable semantics while preserving FIFO ordering and drain-on-pending behavior.

**Owning Crate**: `mempool` [PROPOSED: new crate at `crates/mempool/`]

**Key Types & Traits**:
- `PersistentTxPool` [PROPOSED]
  - High-level `TxSource` implementor wrapping `MempoolStore`
  - Thread-safe via `Arc<Mutex<MempoolStore>>`
- `MempoolStore` [PROPOSED]
  - Raw MDBX wrapper for low-level database operations
  - Auto-increment counter for FIFO key assignment
  - Atomic drain operations (read-all + delete-all + commit)

**Responsibilities**:
- Persist raw EIP-2718 transaction bytes to MDBX database
- Maintain FIFO ordering using auto-increment u64 keys
- Implement drain-on-`pending()` semantics matching `InMemoryTxPool`
- Recover pending transactions on node restart
- Provide concurrent-safe write operations (MDBX single-writer + Mutex)

**Storage Schema** [PROPOSED]:
```
Table: pending_txs
  Key: u64 (auto-increment insertion order)
  Value: Vec<u8> (raw EIP-2718 encoded transaction)

Metadata Key: "counter"
  Value: u64 (next available key)
```

**Dependencies**:
- `app` — for `TxSource` trait definition
- `libmdbx-rs` — raw MDBX bindings (not reth-db, see justification below)
- `parking_lot` — lightweight `Mutex` for interior mutability

**Public API** [PROPOSED]:
```rust
pub struct PersistentTxPool { /* private fields */ }

impl PersistentTxPool {
    pub fn open(path: PathBuf) -> Result<Self, MempoolError>;
}

impl TxSource for PersistentTxPool {
    fn pending(&self) -> Vec<Vec<u8>>;  // Drains DB atomically
    fn push(&self, tx: Vec<u8>);         // Persists with auto-increment key
}
```

**Error Types** [PROPOSED]:
- `MempoolError::DatabaseOpen` — MDBX initialization failure
- `MempoolError::WriteTransaction` — persist operation failure
- `MempoolError::ReadTransaction` — drain operation failure

**Design Rationale** [PROPOSED]:
- **Why raw libmdbx-rs, not reth-db?**
  - [GROUNDED] reth-db's `Tables` enum is vendor-controlled, cannot be extended (`state-reth` uses predefined tables)
  - [PROPOSED] MDBX supports multiple independent databases via separate paths
  - [PROPOSED] Mempool schema is simple (u64 → bytes) — full reth-db abstractions unnecessary
- **Why auto-increment keys?**
  - [GROUNDED] Consensus expects FIFO ordering (`app-evm/src/executor.rs:155` drains oldest-first)
  - [PROPOSED] Cheap inserts — no decoding/hashing overhead (tx validation deferred to consensus)
  - [GROUNDED] `InMemoryTxPool` stores duplicates (no dedup logic) — preserve this behavior

**Crash Recovery Semantics** [PROPOSED]:
- **Normal operation**: push → persist → pending drains → DB empty
- **Crash before pending()**: txs remain in DB → recovered on next startup → returned by next `pending()`
- **Crash after pending() drains**: txs deleted from DB before crash → lost (same as `InMemoryTxPool`)
- **Crash between propose and finalize**: txs lost (accepted risk, matches current behavior)

**Evidence**:
- `state-reth/src/db.rs` — reference MDBX usage pattern via reth-db
- `app/src/tx_source.rs:36` — `InMemoryTxPool` drain semantics to match
- `app-evm/src/executor.rs:155` — consensus FIFO expectation

---

### 3. RPC Domain (rpc-eth crate)

**Bounded Context**: Ethereum JSON-RPC handling. Accepts raw transactions from external clients and submits them to the tx pool via trait interface.

**Owning Crate**: `rpc-eth` (`crates/rpc-eth/`)

**Key Types & Traits**:
- `EthRpcContext` [GROUNDED: `rpc-eth/src/context.rs:14`]
  - [GROUNDED] Current: `tx_pool: Arc<InMemoryTxPool>` (concrete type)
  - [PROPOSED] New: `tx_pool: Arc<dyn TxSource + Send + Sync>` (trait object)
- `EthApiHandler` [GROUNDED: implements JSON-RPC methods]
- `send_raw_transaction` [GROUNDED: entry point for tx submission]

**Responsibilities**:
- Accept raw transactions via `eth_sendRawTransaction` RPC method
- Submit transactions to tx pool via `TxSource::push()`
- Query state, blocks, receipts via other context fields (unchanged)

**Dependencies**:
- `app` — for `TxSource` trait (trait object boundary)
- `state` — for `StateDb` trait object
- `jsonrpsee` — RPC framework

**Public API Changes** [PROPOSED]:
```rust
pub struct EthRpcContext<S: StateDb, B: BlockStorage> {
    tx_pool: Arc<dyn TxSource + Send + Sync>,  // Changed from Arc<InMemoryTxPool>
    state_db: Arc<RwLock<S>>,
    block_storage: Arc<B>,
    receipt_store: ReceiptStore,
    chain_id: u64,
    block_height: AtomicU64,
}

impl<S: StateDb, B: BlockStorage> EthRpcContext<S, B> {
    pub fn new(
        tx_pool: Arc<dyn TxSource + Send + Sync>,  // Updated signature
        // ... other fields unchanged
    ) -> Self { /* ... */ }
}
```

**Error Types**: No new errors (trait object is infallible for push/pending per trait contract)

**Design Rationale** [PROPOSED]:
- **Why trait object instead of generic type parameter?**
  - [GROUNDED] `EvmApplication` already uses trait object pattern (`app-evm/src/executor.rs:48`)
  - [PROPOSED] Avoids propagating type parameter through `EthApiHandler`, `EthApiServer`, `start_rpc_server`
  - [PROPOSED] Trait object overhead negligible for mempool ops (not hot path)
  - [PROPOSED] Matches established pattern in codebase

**Evidence**:
- `rpc-eth/src/context.rs:14` — current concrete type field
- `rpc-eth/src/eth_handler.rs` — RPC method implementations
- `app-evm/src/executor.rs:48` — existing trait object usage pattern

---

### 4. Wiring Domain (whirlpool-node)

**Bounded Context**: Application assembly and dependency injection. Constructs concrete implementations and wires them together via trait boundaries.

**Owning Crate**: `whirlpool-node` (`crates/whirlpool-node/`)

**Key Types & Traits**:
- `main.rs` [GROUNDED] — binary entrypoint with full component wiring
- `PersistingFinalizationSink` [GROUNDED] — wraps consensus finalization

**Responsibilities**:
- Compute storage paths from `persistent_storage_dir` runtime config
- Instantiate `PersistentTxPool::open(mempool_path)`
- Wrap in `Arc<dyn TxSource + Send + Sync>` trait object
- Pass trait object to `EvmApplication::new()` and `EthRpcContext::new()`
- Manage lifecycle (startup, shutdown, error handling)

**Storage Path Strategy** [PROPOSED]:
```
{persistent_storage_dir}/
  ├── state/       (existing: state-reth DB)
  ├── blocks/      (existing: block storage)
  ├── receipts/    (existing: receipt store)
  └── mempool/     (NEW: PersistentTxPool MDBX database)
```

**Dependencies**:
- `app` — for `TxSource` trait
- `app-evm` — for `EvmApplication`
- `rpc-eth` — for `EthRpcContext`
- `mempool` — for `PersistentTxPool` [PROPOSED new dependency]
- `consensus-simplex` — for consensus wiring
- `state-reth` — for state DB
- `commonware` — for runtime

**Wiring Code Changes** [PROPOSED]:
```rust
// Before (current):
let tx_pool = Arc::new(InMemoryTxPool::new());

// After (proposed):
let mempool_path = persistent_storage_dir.join("mempool");
let tx_pool = Arc::new(
    PersistentTxPool::open(mempool_path)
        .expect("failed to open mempool database")
);
let tx_pool_trait: Arc<dyn TxSource + Send + Sync> = tx_pool;

// Pass trait object to consumers (same as before, but now explicit trait object):
let evm_app = EvmApplication::new(tx_pool_trait.clone(), /* ... */);
let rpc_ctx = EthRpcContext::new(tx_pool_trait.clone(), /* ... */);
```

**Error Handling** [PROPOSED]:
- Fatal on `PersistentTxPool::open()` failure (cannot proceed without mempool)
- Log error with diagnostic info (path, permissions, disk space)
- Exit cleanly (no corrupt state)

**Evidence**:
- `whirlpool-node/src/main.rs` — current wiring code for `InMemoryTxPool`
- `whirlpool-node/src/main.rs` — existing persistence path management for state/blocks/receipts

---

### 5. Execution Domain (app-evm crate)

**Bounded Context**: EVM execution and block building. Consumes transactions from tx pool, executes in REVM, produces blocks and receipts.

**Owning Crate**: `app-evm` (`crates/app-evm/`)

**Key Types & Traits**:
- `EvmApplication` [GROUNDED: `app-evm/src/executor.rs`]
  - Stores `Arc<dyn TxSource + Send + Sync>` [GROUNDED: already trait object]
- `propose()` — drains `tx_source.pending()`, builds block
- `verify()` — deterministic replay for consensus validation
- `store_finalized_block()` — persists finalized blocks

**Responsibilities**:
- Drain pending transactions via `TxSource::pending()`
- Decode EIP-2718 encoded transactions
- Execute transactions in REVM, track state changes
- Compute roots (state, receipts, transactions)
- Persist finalized blocks and receipts

**Status**: **UNCHANGED** [GROUNDED]

**Rationale**: `EvmApplication` already uses trait object abstraction (`Arc<dyn TxSource + Send + Sync>` at `app-evm/src/executor.rs:48`). No code changes required — persistence is transparent at this boundary.

**Evidence**:
- `app-evm/src/executor.rs:48` — trait object field definition
- `app-evm/src/executor.rs:155` — `propose()` calls `self.tx_source.pending()`

---

## Cross-Domain Boundaries

### Boundary 1: Trait Contract (app ↔ mempool / rpc-eth / app-evm)

**Interface**: `TxSource` trait

**Contract** [PROPOSED]:
```rust
pub trait TxSource: Send + Sync {
    /// Drain all pending transactions in FIFO order.
    /// After this call, the pool is empty (until new txs pushed).
    fn pending(&self) -> Vec<Vec<u8>>;
    
    /// Submit a raw EIP-2718 encoded transaction to the pool.
    /// No validation performed — deferred to execution layer.
    fn push(&self, tx: Vec<u8>);
}
```

**Invariants**:
- `pending()` returns transactions in FIFO order (insertion order preserved)
- `pending()` drains the pool (subsequent calls return empty until new pushes)
- `push()` is idempotent from caller perspective (duplicates allowed, no errors)
- Thread-safe: multiple concurrent `push()` calls and one `pending()` drain allowed

**Direction**: Bidirectional
- **Push path**: `rpc-eth` → `TxSource::push()` → `mempool`
- **Drain path**: `app-evm` → `TxSource::pending()` → `mempool`

**Error Propagation**: None — trait methods are infallible. Implementors handle errors internally (log + skip tx on persist failure, return empty vec on drain failure).

**Evidence**:
- `app/src/traits.rs:23` — trait definition location
- `app/src/tx_source.rs:36` — `InMemoryTxPool` reference behavior

---

### Boundary 2: Trait Object Boundary (whirlpool-node ↔ rpc-eth / app-evm)

**Interface**: `Arc<dyn TxSource + Send + Sync>`

**Responsibility**: Dependency injection boundary separating concrete implementation from consumers.

**Direction**: Unidirectional (node wiring → consumers)

**Wiring Pattern** [PROPOSED]:
```rust
// whirlpool-node creates and owns concrete instance:
let concrete_pool: Arc<PersistentTxPool> = Arc::new(PersistentTxPool::open(path)?);

// Cast to trait object for injection:
let trait_object: Arc<dyn TxSource + Send + Sync> = concrete_pool;

// Inject into consumers:
let evm = EvmApplication::new(trait_object.clone(), /* ... */);
let rpc = EthRpcContext::new(trait_object, /* ... */);
```

**Lifetime**: `'static` — trait object lives for duration of node process

**Clone Semantics**: `Arc` provides cheap cloning — reference counting, not deep copy

**Evidence**:
- `app-evm/src/executor.rs:48` — existing trait object field
- `whirlpool-node/src/main.rs` — existing Arc-based wiring pattern

---

### Boundary 3: Persistence Abstraction (mempool ↔ libmdbx-rs)

**Interface**: Raw MDBX C API via `libmdbx-rs` bindings

**Internal to `mempool` crate**: This boundary is NOT exposed to other domains.

**MempoolStore Operations** [PROPOSED]:
- `open(path)` → initialize/open MDBX environment + database
- `push(tx)` → write transaction: fetch-add counter, put(key, value), commit
- `drain_pending()` → write transaction: scan all keys ascending, collect values, delete all keys, commit

**Concurrency Model**:
- [GROUNDED] MDBX provides concurrent readers + single writer
- [PROPOSED] `Mutex` guards all `MempoolStore` operations (coarse-grained locking)
- [PROPOSED] Write transactions are short (< 1ms expected) — mutex contention unlikely

**Error Handling** [PROPOSED]:
- `open()` failure → propagate to caller (fatal at node startup)
- `push()` failure → log error, silently drop transaction (preserve infallible trait contract)
- `drain_pending()` failure → log error, return empty vec (preserve infallible trait contract)

**Evidence**:
- `state-reth/src/db.rs` — reference MDBX usage pattern
- libmdbx documentation — concurrent reader + single writer guarantees

---

### Boundary 4: Transaction Lifecycle (rpc-eth → mempool → app-evm → consensus → finalization)

**Flow**:
```
1. Client submits tx via eth_sendRawTransaction
   ↓
2. rpc-eth: EthApiHandler calls ctx.tx_pool.push(raw_bytes)
   ↓
3. mempool: PersistentTxPool persists to MDBX (auto-increment key)
   ↓
4. app-evm: EvmApplication.propose() drains tx_source.pending()
   ↓
5. mempool: PersistentTxPool atomically reads + deletes all txs from MDBX
   ↓
6. app-evm: Decode EIP-2718, execute in REVM, build block
   ↓
7. consensus-simplex: BFT consensus on block proposal
   ↓
8. app-evm: store_finalized_block() persists block + receipts
```

**Crash Recovery Points**:
- **Before step 3 commit**: tx lost (client must retry)
- **After step 3, before step 5**: tx persisted, recovered on restart, included in next proposal
- **After step 5, before step 8**: tx drained from mempool but block not finalized → **tx lost** (same as current `InMemoryTxPool` behavior)

**Future Enhancement** [PROPOSED — Out of Scope]:
- Add `proposed_txs` table to track lifecycle: `submitted` → `proposed` → `finalized`
- On crash, re-queue `proposed` txs back to `pending_txs`
- Requires finalization callback integration

**Evidence**:
- `rpc-eth/src/eth_handler.rs` — RPC submission entry point
- `app-evm/src/executor.rs:155` — consensus proposal drain point
- `whirlpool-node/src/main.rs` — `PersistingFinalizationSink` finalization hook

---

## Wiring Table

| Capability | Owning Crate | Trait Interface | Provider | Config | Evidence |
|------------|--------------|-----------------|----------|--------|----------|
| Transaction submission | `rpc-eth` | `TxSource::push()` | `PersistentTxPool` | Trait object via constructor | `rpc-eth/src/context.rs:14` |
| Transaction drain | `app-evm` | `TxSource::pending()` | `PersistentTxPool` | Trait object via constructor | `app-evm/src/executor.rs:48` |
| Persistent storage | `mempool` | `TxSource` impl | `MempoolStore` (internal) | Path from node wiring | [PROPOSED] `mempool/src/persistent.rs` |
| Storage backend | `mempool` | Raw MDBX API | `libmdbx-rs` crate | MDBX environment args | [PROPOSED] `mempool/src/store.rs` |
| Trait definition | `app` | `pub trait TxSource` | N/A (foundational) | None | `app/src/traits.rs:23` |
| Dependency injection | `whirlpool-node` | `Arc<dyn TxSource + Send + Sync>` | Node main | Storage path from runtime | `whirlpool-node/src/main.rs` |
| Reference impl (tests) | `app` | `TxSource` impl | `InMemoryTxPool` | None (in-memory) | `app/src/tx_source.rs:30-45` |
| State persistence | `state-reth` | `StateDb` trait | `RethStateDb` | Separate MDBX path | `state-reth/src/db.rs` |

---

## Error Propagation Strategy

### Design Principle [PROPOSED]
Mempool operations are **best-effort** at runtime — failures should not crash the node. Errors are logged and degraded gracefully.

### Error Boundaries

#### 1. Startup (Fatal Errors)
- **Trigger**: `PersistentTxPool::open(path)` failure
- **Handling**: Propagate error to `main()`, log diagnostic, exit process
- **Rationale**: Cannot operate without mempool — better to fail fast than run with broken persistence

#### 2. Runtime Push (Non-Fatal Errors)
- **Trigger**: MDBX write transaction failure during `push()`
- **Handling**: Log error, drop transaction, return success to caller
- **Rationale**: Preserves `TxSource::push()` infallible contract, avoids RPC error cascade
- **Degradation**: Transaction lost, client must retry (same as network loss)

#### 3. Runtime Drain (Non-Fatal Errors)
- **Trigger**: MDBX read/delete transaction failure during `pending()`
- **Handling**: Log error, return empty vec to caller
- **Rationale**: Preserves `TxSource::pending()` infallible contract, consensus proposes empty block
- **Degradation**: Temporary unavailability, next proposal may succeed

### Logging Strategy [PROPOSED]
```rust
// Example error handling in MempoolStore:
fn push(&self, tx: Vec<u8>) {
    match self.write_tx_internal(tx) {
        Ok(_) => {},
        Err(e) => {
            tracing::error!(
                error = ?e,
                tx_len = tx.len(),
                "Failed to persist transaction to mempool DB, dropping tx"
            );
        }
    }
}
```

### Observability [PROPOSED — Future Enhancement]
- Metrics: `mempool_push_errors_total`, `mempool_drain_errors_total`
- Alerts: Sustained error rate → indicates storage corruption, disk full, or permission issues

---

## Testing Strategy

### Unit Tests (mempool crate)
- `push → pending drains → empty` [PROPOSED]
- `push → restart (drop + recreate) → pending recovers txs` [PROPOSED]
- `concurrent push from multiple threads` [PROPOSED]
- `drain with no txs → empty vec` [PROPOSED]
- `FIFO ordering: push A, B, C → pending returns [A, B, C]` [PROPOSED]

### Integration Tests (app-evm or whirlpool-node)
- `RPC submit → propose drains → block contains tx` [PROPOSED]
- `RPC submit → restart → propose drains → block contains tx` [PROPOSED]
- `RPC submit → propose → crash before finalize → tx lost (expected behavior)` [PROPOSED — documents crash semantics]

### Trait Contract Tests (app crate)
- Mock `TxSource` implementations for negative testing [GROUNDED: existing test helpers]
- Verify `InMemoryTxPool` and `PersistentTxPool` share same trait semantics [PROPOSED]

---

## Unknowns and Blockers

### Unknowns
- `UNKNOWN`: Actual performance overhead of MDBX writes on target hardware (expect <1ms, but unmeasured)
- `UNKNOWN`: Disk space growth rate for mempool DB under realistic load (depends on tx volume + drain frequency)

### Resolved Questions
- ✅ Can MDBX coexist with reth-db state DB? → Yes, via separate directory paths [GROUNDED: multi-DB support confirmed]
- ✅ Does `EvmApplication` need changes? → No, already uses trait object [GROUNDED: `app-evm/src/executor.rs:48`]
- ✅ How to handle deduplication? → Not handled in MVP (preserve `InMemoryTxPool` behavior) [PROPOSED: future enhancement]

### No Blockers
All design decisions resolved. Implementation can proceed through Phase 1-4 as defined in STRATEGY.md.

---

## References

- **INTENT.md** — Original requirements and success criteria
- **STRATEGY.md** — Detailed implementation strategy, phase ordering, technology justification
- **SHARED_CONTEXT.md** — Existing codebase patterns, transaction lifecycle, persistence examples
- **CRATES.md** — Crate-by-crate change inventory and dependency graph

---

## Document Metadata

- **Version**: 1.0
- **Status**: Design complete, ready for implementation
- **Last Updated**: 2026-03-07
- **Phase**: Pre-implementation (Phase 0)
