# rpc-eth

## Purpose

[GROUNDED] Provides Ethereum JSON-RPC server implementation for external client interaction. Handles `eth_*` methods (transaction submission, state queries, block/receipt retrieval). [PROPOSED] Generified to accept trait object `Arc<dyn TxSource>` instead of concrete `Arc<InMemoryTxPool>`, enabling pluggable transaction pool implementations (persistent vs in-memory).

## Public API

[GROUNDED] Current API (`rpc-eth/src/context.rs:12-51`):

```rust
pub struct EthRpcContext<S: StateDb, B: BlockStorage> {
    pub tx_pool: Arc<InMemoryTxPool>,       // [GROUNDED] line 13, concrete type
    pub state_db: Arc<RwLock<S>>,
    pub block_storage: Arc<B>,
    pub receipt_store: Arc<ReceiptStore>,
    pub chain_id: u64,
    pub block_height: Arc<AtomicU64>,
}

impl<S: StateDb, B: BlockStorage> EthRpcContext<S, B> {
    pub fn new(
        tx_pool: Arc<InMemoryTxPool>,       // [GROUNDED] line 37, concrete type
        state_db: Arc<RwLock<S>>,
        block_storage: Arc<B>,
        chain_id: u64,
    ) -> Self;
}
```

[PROPOSED] Generified API:

```rust
pub struct EthRpcContext<S: StateDb, B: BlockStorage> {
    pub tx_pool: Arc<dyn TxSource + Send + Sync>,  // Changed to trait object
    pub state_db: Arc<RwLock<S>>,                   // Unchanged
    pub block_storage: Arc<B>,                      // Unchanged
    pub receipt_store: Arc<ReceiptStore>,           // Unchanged
    pub chain_id: u64,                              // Unchanged
    pub block_height: Arc<AtomicU64>,               // Unchanged
}

impl<S: StateDb, B: BlockStorage> EthRpcContext<S, B> {
    pub fn new(
        tx_pool: Arc<dyn TxSource + Send + Sync>,  // Changed to trait object
        state_db: Arc<RwLock<S>>,
        block_storage: Arc<B>,
        chain_id: u64,
    ) -> Self;
}
```

**Clone Implementation**: [GROUNDED] Manual `Clone` impl exists (line 22-33) to avoid requiring `B: Clone`. [PROPOSED] No change needed — `Arc<dyn TxSource>` is `Clone` via `Arc` cloning (cheap reference count increment).

## Config

[GROUNDED] No configuration — `EthRpcContext` is pure dependency injection container. RPC server binding address configured in `whirlpool-node` wiring (`config::RPC_BIND_ADDR`).

## Internal Modules

[GROUNDED] Module structure (`crates/rpc-eth/src/`):

- `lib.rs` — re-exports public types
- `context.rs` — `EthRpcContext` struct (line 12)
- `eth_handler.rs` — `EthApiHandler` implementing JSON-RPC methods
- `server.rs` — `start_rpc_server()` function, jsonrpsee server setup
- `receipt_store.rs` — `ReceiptStore` in-memory cache
- `convert.rs` — type conversions (reth types ↔ RPC types)

[PROPOSED] No new modules — change confined to `context.rs`.

## Primary Flows

### Flow 1: Transaction Submission [GROUNDED with PROPOSED change]

[GROUNDED] Current flow (`rpc-eth/src/eth_handler.rs`, `send_raw_transaction` method):
```
eth_sendRawTransaction RPC call
  → EthApiHandler::send_raw_transaction(raw_tx: Bytes)
  → ctx.tx_pool.push(raw_tx.to_vec())  // [GROUNDED] calls InMemoryTxPool::push
  → Return tx hash to client
```

[PROPOSED] After generification:
```
eth_sendRawTransaction RPC call
  → EthApiHandler::send_raw_transaction(raw_tx: Bytes)
  → ctx.tx_pool.push(raw_tx.to_vec())  // Now calls TxSource::push trait method
  → Return tx hash to client
```

**Change**: Method call `ctx.tx_pool.push()` unchanged syntactically — dynamic dispatch via trait object instead of concrete type. No logic modification.

### Flow 2: State Queries [GROUNDED]

[GROUNDED] `eth_getBalance`, `eth_getCode`, `eth_getTransactionCount`:
```
RPC method call
  → EthApiHandler method
  → ctx.state_db.read().unwrap()
  → StateDb trait methods (get_account, get_code, get_nonce)
  → Return result to client
```

**Status**: UNCHANGED — no tx pool interaction.

### Flow 3: Block/Receipt Queries [GROUNDED]

[GROUNDED] `eth_getBlockByNumber`, `eth_getTransactionReceipt`:
```
RPC method call
  → EthApiHandler method
  → ctx.block_storage (for blocks) or ctx.receipt_store (for receipts)
  → Query storage, convert to RPC types
  → Return result to client
```

**Status**: UNCHANGED — no tx pool interaction.

## Dependencies

[GROUNDED] Current dependencies (`rpc-eth/Cargo.toml`):

- `app` — for `InMemoryTxPool` concrete type
- `state` — for `StateDb`, `BlockStorage` traits
- `jsonrpsee` — RPC framework
- `serde` — serialization
- `tokio` — async runtime

[PROPOSED] No new dependencies. Import changes:

**Before**:
```rust
use app::tx_source::InMemoryTxPool;
```

**After**:
```rust
use app::traits::TxSource;  // Import trait, not concrete type
```

## Error Types

[GROUNDED] RPC errors (existing, unchanged):

- `INTERNAL_ERROR` (-32603): State query failure, storage unavailable
- `INVALID_PARAMS` (-32602): Malformed request parameters
- `METHOD_NOT_FOUND` (-32601): Unsupported RPC method

[PROPOSED] No new errors — `TxSource::push()` is infallible per trait contract. Transaction submission always returns success to client (tx may be dropped internally on persist failure, client must monitor for inclusion).

## Invariants and Contracts

[GROUNDED] Existing invariants:

1. **Concurrent access**: `EthRpcContext` is `Clone` (line 22) and shared across RPC handler tasks. All fields use `Arc` for safe concurrent access.
2. **State consistency**: `state_db` wrapped in `RwLock` for multi-reader + single-writer access (state updates during block finalization).
3. **Thread safety**: All fields satisfy `Send + Sync` (required by jsonrpsee server).

[PROPOSED] New invariants with trait object:

1. **Trait object bounds**: `Arc<dyn TxSource + Send + Sync>` required — `TxSource` trait extended with explicit bounds (see `app.md`).
2. **Polymorphism**: RPC layer agnostic to tx pool implementation — works with `InMemoryTxPool`, `PersistentTxPool`, or future variants.
3. **No type parameter propagation**: Generic type parameter avoided — `EthRpcContext` remains generic only over `S: StateDb` and `B: BlockStorage` (no `T: TxSource` parameter).

## What Changes from Current Code

### Change 1: EthRpcContext Field Type [PROPOSED]

**File**: `rpc-eth/src/context.rs`

**Before** (line 13):
```rust
pub tx_pool: Arc<InMemoryTxPool>,
```

**After**:
```rust
pub tx_pool: Arc<dyn TxSource + Send + Sync>,
```

**Rationale**: Accept any `TxSource` implementor, not just `InMemoryTxPool`. Enables persistent pool integration without RPC layer changes.

### Change 2: EthRpcContext Constructor [PROPOSED]

**File**: `rpc-eth/src/context.rs`

**Before** (line 36-50):
```rust
impl<S: StateDb, B: BlockStorage> EthRpcContext<S, B> {
    pub fn new(
        tx_pool: Arc<InMemoryTxPool>,       // line 37
        state_db: Arc<RwLock<S>>,
        block_storage: Arc<B>,
        chain_id: u64,
    ) -> Self {
        Self {
            tx_pool,                         // line 43
            state_db,
            block_storage,
            receipt_store: Arc::new(ReceiptStore::new()),
            chain_id,
            block_height: Arc::new(AtomicU64::new(0)),
        }
    }
}
```

**After**:
```rust
impl<S: StateDb, B: BlockStorage> EthRpcContext<S, B> {
    pub fn new(
        tx_pool: Arc<dyn TxSource + Send + Sync>,  // Changed parameter type
        state_db: Arc<RwLock<S>>,
        block_storage: Arc<B>,
        chain_id: u64,
    ) -> Self {
        Self {
            tx_pool,                                // No change (type inference)
            state_db,
            block_storage,
            receipt_store: Arc::new(ReceiptStore::new()),
            chain_id,
            block_height: Arc::new(AtomicU64::new(0)),
        }
    }
}
```

**Impact**: All callers of `EthRpcContext::new()` must pass trait object. In practice, only `whirlpool-node/src/main.rs` calls this (line 127) — updated in integration phase.

### Change 3: Import Statement [PROPOSED]

**File**: `rpc-eth/src/context.rs`

**Before** (line 1):
```rust
use app::tx_source::InMemoryTxPool;
```

**After**:
```rust
use app::traits::TxSource;
```

**Rationale**: Import trait definition, not concrete type. Concrete type instantiated in node wiring layer.

### Change 4: Clone Implementation [UNCHANGED]

**File**: `rpc-eth/src/context.rs`

**Current** (line 21-33):
```rust
impl<S: StateDb, B: BlockStorage> Clone for EthRpcContext<S, B> {
    fn clone(&self) -> Self {
        Self {
            tx_pool: self.tx_pool.clone(),  // Arc clone — works for trait object
            state_db: self.state_db.clone(),
            block_storage: self.block_storage.clone(),
            receipt_store: self.receipt_store.clone(),
            chain_id: self.chain_id,
            block_height: self.block_height.clone(),
        }
    }
}
```

**Status**: No change required — `Arc<dyn TxSource>` is `Clone` (clones the `Arc`, not the underlying type).

### Change 5: RPC Handler Usage [UNCHANGED]

**File**: `rpc-eth/src/eth_handler.rs`

**Current** (conceptual, exact location varies):
```rust
ctx.tx_pool.push(raw_tx.to_vec());
```

**Status**: No change — method call syntax identical. Dynamic dispatch via trait object instead of concrete type (transparent to caller).

## Open Questions

None — generification is mechanical, no semantic changes to RPC behavior.
