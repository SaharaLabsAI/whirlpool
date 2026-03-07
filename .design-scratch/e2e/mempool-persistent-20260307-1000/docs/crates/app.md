# app

## Purpose

[GROUNDED] Defines application-layer traits and shared types for consensus integration. Provides `TxSource` trait contract for transaction pool abstraction and reference implementations for testing. [PROPOSED] Extended to include `push()` method on `TxSource` trait, enabling trait object usage in RPC layer.

## Public API

[GROUNDED] Current API (`app/src/traits.rs:23`, `app/src/tx_source.rs:30-52`):

```rust
pub trait TxSource {
    fn pending(&self) -> Vec<Vec<u8>>;  // [GROUNDED] line 24
}

pub struct InMemoryTxPool { /* Mutex<Vec<Vec<u8>>> */ }
impl InMemoryTxPool {
    pub fn new() -> Self;                 // [GROUNDED] line 26
    pub fn push(&self, tx: Vec<u8>);      // [GROUNDED] line 33, NOT in trait
}

pub struct NoopTxSource;  // [GROUNDED] line 5
```

[PROPOSED] Extended API (trait modification):

```rust
pub trait TxSource: Send + Sync {  // Add trait bounds explicitly
    /// Drain all pending transactions in FIFO order.
    /// After this call, the pool is empty until new pushes.
    fn pending(&self) -> Vec<Vec<u8>>;
    
    /// Submit a raw EIP-2718 encoded transaction to the pool.
    /// No validation performed — deferred to execution layer.
    fn push(&self, tx: Vec<u8>);  // NEW method
}
```

**Trait Bounds**: [PROPOSED] `Send + Sync` required for `Arc<dyn TxSource>` trait object usage (already implicit in `InMemoryTxPool` implementation via `Mutex`).

## Config

[GROUNDED] No configuration — `InMemoryTxPool` and `NoopTxSource` have hardcoded behavior (in-memory FIFO, empty stub).

## Internal Modules

[GROUNDED] Module structure (`crates/app/src/`):

- `lib.rs` — re-exports public types
- `traits.rs` — `Application` trait (line 3), `TxSource` trait (line 23)
- `tx_source.rs` — `InMemoryTxPool` (line 20), `NoopTxSource` (line 5)
- `types.rs` — `EvmBlock`, `ExecutionResult` (consensus-facing types)

[PROPOSED] No new modules — changes confined to existing trait definition.

## Primary Flows

### Flow 1: InMemoryTxPool Push and Drain [GROUNDED]
```
InMemoryTxPool::push(tx)
  → Acquire Mutex lock (line 34)
  → Vec::push(tx)
  → Release mutex

TxSource::pending()
  → Acquire Mutex lock (line 50)
  → std::mem::take(&mut *guard) — drain Vec, leave empty Vec
  → Release mutex
  → Return drained Vec in FIFO order
```

**Evidence**: `app/src/tx_source.rs:33-35` (push method), `app/src/tx_source.rs:49-51` (pending method).

**Semantics**: [GROUNDED] Drain-on-pending — each transaction returned at most once (line 18-19 comment).

### Flow 2: Trait Extension Impact [PROPOSED]
```
Extend TxSource trait with push()
  → Update InMemoryTxPool impl:
      Already has push() as inherent method (line 33)
      Lift into TxSource impl block — no logic change
  → Update NoopTxSource impl:
      Add empty push() method (no-op, discard tx)
  → Update test mocks:
      app-evm/tests/integration.rs: MockTxSource
      rpc-eth test helpers
```

**Breaking Change**: [PROPOSED] Trait extension breaks compilation for all implementors. Mitigated by updating all in-tree implementors atomically in same commit (no external implementors exist per workspace analysis).

## Dependencies

[GROUNDED] No dependencies — foundational crate at bottom of dependency graph.

[GROUNDED] Dependent crates:
- `app-evm` — imports `TxSource` trait for `EvmApplication` constructor
- `rpc-eth` — [PROPOSED] will import `TxSource` trait object instead of `InMemoryTxPool` concrete type
- `whirlpool-node` — imports `InMemoryTxPool` for wiring (changes to `mempool::PersistentTxPool` in integration phase)
- `mempool` — [PROPOSED] new crate, imports `TxSource` trait for `PersistentTxPool` implementation

## Error Types

[GROUNDED] No errors exposed — `TxSource` trait methods are infallible. `InMemoryTxPool` uses `Mutex` with `expect("lock poisoned")` on lock acquisition (panics on poison, acceptable for lock failures).

## Invariants and Contracts

[GROUNDED] Existing `TxSource` contract (implicit, documented in comments):

1. **FIFO ordering**: `pending()` returns transactions in insertion order (enforced by `Vec` in `InMemoryTxPool`, line 48).
2. **Drain semantics**: `pending()` drains the pool — subsequent calls return empty until new pushes (line 47 comment, line 50 implementation).
3. **Thread safety**: `InMemoryTxPool` uses `Mutex` for concurrent access (line 21), safe for multiple push + single drain.

[PROPOSED] Extended contract with `push()` addition:

1. **Push infallibility**: `push()` never panics or returns errors (implementors handle failures internally).
2. **Push idempotence** (from caller perspective): Multiple identical `push()` calls allowed, no errors (duplicates stored, not deduplicated).
3. **Concurrent safety**: Multiple threads may call `push()` concurrently; single thread calls `pending()` (consensus assumption).

[PROPOSED] Trait bounds contract:

- `Send + Sync` required for trait object `Arc<dyn TxSource + Send + Sync>` usage in `EthRpcContext` and `EvmApplication`.
- Implementors must be thread-safe (enforced by `Mutex` in `InMemoryTxPool`, no-op in `NoopTxSource`).

## What Changes from Current Code

### Change 1: TxSource Trait Extension [PROPOSED]

**File**: `app/src/traits.rs`

**Before** (line 23-25):
```rust
pub trait TxSource {
    fn pending(&self) -> Vec<Vec<u8>>;
}
```

**After**:
```rust
pub trait TxSource: Send + Sync {
    fn pending(&self) -> Vec<Vec<u8>>;
    fn push(&self, tx: Vec<u8>);
}
```

**Rationale**: Enable trait object usage in RPC layer (`Arc<dyn TxSource>`). RPC needs `push()` to submit transactions; trait must include method for trait object polymorphism.

### Change 2: InMemoryTxPool Trait Impl [PROPOSED]

**File**: `app/src/tx_source.rs`

**Before** (line 44-52):
```rust
impl TxSource for InMemoryTxPool {
    fn pending(&self) -> Vec<Vec<u8>> {
        std::mem::take(&mut *self.txs.lock().expect("tx pool lock poisoned"))
    }
}
```

**After**:
```rust
impl TxSource for InMemoryTxPool {
    fn pending(&self) -> Vec<Vec<u8>> {
        std::mem::take(&mut *self.txs.lock().expect("tx pool lock poisoned"))
    }
    
    fn push(&self, tx: Vec<u8>) {
        self.txs.lock().expect("tx pool lock poisoned").push(tx);
    }
}
```

**Note**: `push()` already exists as inherent method (line 33-35) — no logic change, just move into trait impl block.

### Change 3: NoopTxSource Trait Impl [PROPOSED]

**File**: `app/src/tx_source.rs`

**Before** (line 7-11):
```rust
impl TxSource for NoopTxSource {
    fn pending(&self) -> Vec<Vec<u8>> {
        Vec::new()
    }
}
```

**After**:
```rust
impl TxSource for NoopTxSource {
    fn pending(&self) -> Vec<Vec<u8>> {
        Vec::new()
    }
    
    fn push(&self, tx: Vec<u8>) {
        // No-op: discard transaction (test stub)
    }
}
```

**Rationale**: Test stub for scenarios where transaction pool is not relevant (e.g., state-only tests).

### Change 4: Test Mocks [PROPOSED]

**Files**: `app-evm/tests/integration.rs`, `rpc-eth/src/eth_handler.rs` test helpers

**Required**: Any `MockTxSource` or test helpers implementing `TxSource` must add `push()` method (empty impl acceptable for tests not exercising push path).

**Example**:
```rust
struct MockTxSource { /* ... */ }
impl TxSource for MockTxSource {
    fn pending(&self) -> Vec<Vec<u8>> { /* ... */ }
    fn push(&self, tx: Vec<u8>) { /* no-op or mock recording */ }
}
```

## Open Questions

None — trait extension is straightforward, all implementors identified and updated.
