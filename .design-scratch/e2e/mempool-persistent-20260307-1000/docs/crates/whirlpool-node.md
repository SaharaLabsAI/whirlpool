# whirlpool-node

## Purpose

[GROUNDED] Binary entrypoint for Whirlpool consensus node. Assembles all components (state DB, block storage, tx pool, EVM executor, consensus engine, RPC server) and manages their lifecycle. [PROPOSED] Modified to wire `PersistentTxPool` instead of `InMemoryTxPool`, compute mempool storage path from runtime persistent directory, and pass trait object to consumers.

## Public API

[GROUNDED] Binary crate — no public library API. Exposes command-line interface (currently hardcoded configuration, no CLI parsing).

[GROUNDED] Configuration constants (`whirlpool-node/src/config.rs`, `whirlpool-node/src/main.rs`):

- `APPLICATION_NAMESPACE` — network isolation namespace (line 25)
- `DEFAULT_DB_PATH` — state database path (line 29)
- `DEFAULT_RUNTIME_STORAGE_DIR` — consensus journal path (line 32)
- `RPC_BIND_ADDR` — JSON-RPC server address (referenced line 129)
- `VALIDATOR_SEED` — deterministic validator key (development only)

[PROPOSED] New storage path computation:

```rust
const MEMPOOL_SUBDIR: &str = "mempool";

// Compute mempool path from runtime storage directory
let mempool_path = PathBuf::from(DEFAULT_RUNTIME_STORAGE_DIR).join(MEMPOOL_SUBDIR);
```

## Config

[GROUNDED] Current storage layout:

```
data/
  ├── state/         (state-reth MDBX database, line 70)
  ├── runtime/       (consensus journal, line 48)
```

[PROPOSED] Extended storage layout:

```
data/
  ├── state/         (state-reth MDBX database, UNCHANGED)
  ├── runtime/       (consensus journal, UNCHANGED)
      └── mempool/   (NEW: PersistentTxPool MDBX database)
```

**Rationale**: Nest mempool under runtime directory (both managed by commonware runtime lifetime). Alternative: sibling `data/mempool/` (cleaner separation, but requires additional path constant).

## Internal Modules

[GROUNDED] Module structure (`crates/whirlpool-node/src/`):

- `main.rs` — binary entrypoint, component wiring (line 34-144)
- `config.rs` — configuration constants
- `persisting_sink.rs` — `PersistingFinalizationSink` wrapper for block persistence (line 22)
- `lib.rs` — re-exports for internal use

[PROPOSED] No new modules — changes confined to `main.rs` wiring logic.

## Primary Flows

### Flow 1: Node Startup [GROUNDED with PROPOSED changes]

[GROUNDED] Current flow (`main.rs:34-144`):

```
1. Initialize tracing (line 36-41)
2. Create commonware runtime with persistent storage (line 47-49)
3. Generate validator keypair (line 55-56)
4. Setup network provider (line 62-67)
5. Open state database (line 70-75)
6. Recover chain tip from persistent state (line 78-82)
7. Create InMemoryTxPool (line 108)
8. Create EvmApplication with tx pool (line 109)
9. Create consensus engine (line 120-122)
10. Start RPC server (line 126-134)
11. Wait indefinitely (line 142)
```

[PROPOSED] Modified flow (steps 7-8 changed):

```
1. Initialize tracing (UNCHANGED)
2. Create commonware runtime with persistent storage (UNCHANGED)
3. Generate validator keypair (UNCHANGED)
4. Setup network provider (UNCHANGED)
5. Open state database (UNCHANGED)
6. Recover chain tip from persistent state (UNCHANGED)
7. Open PersistentTxPool (NEW):
     let mempool_path = PathBuf::from(DEFAULT_RUNTIME_STORAGE_DIR).join("mempool");
     let tx_pool = Arc::new(
         PersistentTxPool::open(mempool_path)
             .expect("failed to open mempool database")
     );
     let tx_pool_trait: Arc<dyn TxSource + Send + Sync> = tx_pool;
8. Create EvmApplication with trait object (MODIFIED):
     let evm_app = EvmApplication::new(evm_config, state_db.clone(), tx_pool_trait.clone());
9. Create consensus engine (UNCHANGED)
10. Start RPC server (MODIFIED):
     let rpc_ctx = EthRpcContext::new(tx_pool_trait, state_db, block_storage, SAHARA_CHAIN_ID);
11. Wait indefinitely (UNCHANGED)
```

**Error Handling**: [PROPOSED] `PersistentTxPool::open()` failure is fatal — log error with diagnostic info (path, permissions), exit process cleanly. Cannot proceed without mempool.

### Flow 2: Transaction Pool Wiring [GROUNDED with PROPOSED changes]

[GROUNDED] Current wiring (`main.rs:108-109`):

```rust
let tx_pool = Arc::new(InMemoryTxPool::new());
let evm_app = EvmApplication::new(evm_config, state_db.clone(), tx_pool.clone());
```

[PROPOSED] New wiring:

```rust
use mempool::PersistentTxPool;  // New import

let mempool_path = PathBuf::from(DEFAULT_RUNTIME_STORAGE_DIR).join("mempool");
let tx_pool_concrete = Arc::new(
    PersistentTxPool::open(mempool_path)
        .expect("failed to open mempool database")
);
let tx_pool: Arc<dyn TxSource + Send + Sync> = tx_pool_concrete;

let evm_app = EvmApplication::new(evm_config, state_db.clone(), tx_pool.clone());
```

**Type Coercion**: Explicit trait object cast for clarity (Rust can infer, but explicit cast documents intent).

### Flow 3: RPC Context Wiring [GROUNDED with PROPOSED changes]

[GROUNDED] Current wiring (`main.rs:126-128`):

```rust
let mut rpc_ctx = rpc::context::EthRpcContext::new(
    tx_pool,           // Arc<InMemoryTxPool>
    state_db,
    block_storage,
    SAHARA_CHAIN_ID
);
```

[PROPOSED] New wiring:

```rust
let mut rpc_ctx = rpc::context::EthRpcContext::new(
    tx_pool.clone(),   // Arc<dyn TxSource + Send + Sync>
    state_db,
    block_storage,
    SAHARA_CHAIN_ID
);
```

**Change**: `tx_pool` now trait object — no syntax change in call site, type inference handles it.

### Flow 4: Crash Recovery [PROPOSED]

```
Node crash (power loss, OOM kill, panic)
  → Restart node binary
  → Commonware runtime opens persistent journal (EXISTING)
  → state-reth opens state DB, recovers chain tip (EXISTING, line 70-82)
  → PersistentTxPool::open(mempool_path) (NEW):
      → Open existing MDBX database
      → Scan for max key, resume counter
      → Unproposed txs remain in database
  → EvmApplication.propose() drains recovered txs (NEW):
      → TxSource::pending() returns persisted txs
      → Consensus proposes block with recovered txs
```

**Semantics**: Transactions submitted before crash but not yet proposed are recovered. Transactions drained by `pending()` before crash are lost (same as `InMemoryTxPool`).

## Dependencies

[GROUNDED] Current dependencies (`whirlpool-node/Cargo.toml`):

- `app` — for `InMemoryTxPool`, `ApplicationAdapter`
- `app-evm` — for `EvmApplication`, `build_sahara_chain_spec`
- `consensus` — for `ConsensusEngine` trait
- `consensus-simplex` — for `CommonwareEngine`, `FinalizationSink`
- `p2p-commonware` — for `CommonwareNetworkProviderBuilder`
- `rpc-eth` — for `EthRpcContext`, `start_rpc_server`
- `state-reth` — for `open_state_db`
- `state` — for `BlockStorage` trait
- `commonware-runtime` — for `tokio::Runner`
- `commonware-cryptography` — for `ed25519::PrivateKey`
- `tracing` — for logging

[PROPOSED] New dependency:

- `mempool` — for `PersistentTxPool`

[PROPOSED] Import changes:

**Add**:
```rust
use mempool::PersistentTxPool;
use app::traits::TxSource;  // For trait object type annotation
```

**Remove** (implicit, `InMemoryTxPool` no longer used):
```rust
// use app::InMemoryTxPool;  (currently imported via app re-export, line 3)
```

## Error Types

[GROUNDED] Current error handling:

- `expect()` on database initialization failures (line 73) — fatal error, panic with message
- `expect()` on consensus engine start (line 122) — fatal error
- `expect()` on RPC server start (line 134) — fatal error

[PROPOSED] New error handling:

```rust
let tx_pool = Arc::new(
    PersistentTxPool::open(mempool_path)
        .unwrap_or_else(|e| {
            tracing::error!(
                error = ?e,
                path = ?mempool_path,
                "Failed to open mempool database"
            );
            std::process::exit(1);
        })
);
```

**Alternative**: Use `expect("failed to open mempool database")` for consistency with existing error handling (line 73, 122, 134). Recommended for simplicity.

## Invariants and Contracts

[GROUNDED] Existing invariants:

1. **Single-threaded startup**: All initialization happens sequentially in `main()` — no concurrent access during setup.
2. **Arc-based sharing**: All shared resources (state DB, tx pool, block storage) wrapped in `Arc` for concurrent access across consensus, RPC, and finalization threads.
3. **Persistent storage paths**: All persistence uses absolute or workspace-relative paths (no temp directories for production data).

[PROPOSED] New invariants:

1. **Storage path isolation**: Mempool DB path must not collide with state DB path or consensus journal path. Enforced by nesting under separate subdirectories (`data/state/`, `data/runtime/mempool/`).
2. **Trait object injection**: Consumers receive `Arc<dyn TxSource + Send + Sync>` trait object — no knowledge of concrete `PersistentTxPool` type outside wiring layer.
3. **Fatal startup failures**: All database initialization failures (state, mempool) exit process immediately — no partial initialization or fallback to in-memory mode.

## What Changes from Current Code

### Change 1: Import Statements [PROPOSED]

**File**: `whirlpool-node/src/main.rs`

**Before** (line 3):
```rust
use app::{ApplicationAdapter, InMemoryTxPool};
```

**After**:
```rust
use app::{ApplicationAdapter, traits::TxSource};
use mempool::PersistentTxPool;
```

**Rationale**: Import `TxSource` trait for trait object type annotation, import `PersistentTxPool` concrete type for instantiation.

### Change 2: Tx Pool Initialization [PROPOSED]

**File**: `whirlpool-node/src/main.rs`

**Before** (line 108):
```rust
let tx_pool = Arc::new(InMemoryTxPool::new());
```

**After**:
```rust
let mempool_path = PathBuf::from(DEFAULT_RUNTIME_STORAGE_DIR).join("mempool");
let tx_pool = Arc::new(
    PersistentTxPool::open(mempool_path)
        .expect("failed to open mempool database")
) as Arc<dyn TxSource + Send + Sync>;
```

**Alternative** (explicit variable for clarity):
```rust
let mempool_path = PathBuf::from(DEFAULT_RUNTIME_STORAGE_DIR).join("mempool");
let tx_pool_concrete = Arc::new(
    PersistentTxPool::open(mempool_path)
        .expect("failed to open mempool database")
);
let tx_pool: Arc<dyn TxSource + Send + Sync> = tx_pool_concrete;
```

### Change 3: EvmApplication Wiring [UNCHANGED]

**File**: `whirlpool-node/src/main.rs`

**Current** (line 109):
```rust
let evm_app = EvmApplication::new(evm_config, state_db.clone(), tx_pool.clone());
```

**Status**: No change — `tx_pool` now trait object, but syntax identical. `EvmApplication::new` already accepts trait object (see `app-evm/src/executor.rs:48`).

### Change 4: RPC Context Wiring [MODIFIED]

**File**: `whirlpool-node/src/main.rs`

**Before** (line 126-128):
```rust
let mut rpc_ctx =
    rpc::context::EthRpcContext::new(tx_pool, state_db, block_storage, SAHARA_CHAIN_ID);
```

**After**:
```rust
let mut rpc_ctx =
    rpc::context::EthRpcContext::new(tx_pool.clone(), state_db, block_storage, SAHARA_CHAIN_ID);
```

**Note**: `.clone()` added because `tx_pool` consumed by `EvmApplication::new` earlier. Alternative: clone before `EvmApplication::new`, pass clones to both.

### Change 5: Cargo.toml Dependency [PROPOSED]

**File**: `whirlpool-node/Cargo.toml`

**Add**:
```toml
[dependencies]
mempool = { path = "../mempool" }
```

### Change 6: Storage Path Documentation [PROPOSED]

**File**: `whirlpool-node/src/main.rs`

**Add comment** (before mempool path computation):
```rust
// Create persistent transaction pool (survives restarts)
// Storage path: {DEFAULT_RUNTIME_STORAGE_DIR}/mempool/
let mempool_path = PathBuf::from(DEFAULT_RUNTIME_STORAGE_DIR).join("mempool");
```

## Open Questions

[PROPOSED] Design decisions for consideration:

1. **Mempool path location**: Nest under `data/runtime/mempool/` or sibling `data/mempool/`?
   - **Recommendation**: Sibling `data/mempool/` for consistency with `data/state/` (both are persistent application data, not runtime metadata).
   - **Alternative**: Nest under runtime if mempool is considered ephemeral compared to state (but persistence implies it's not truly ephemeral).

2. **Error handling style**: `expect()` vs explicit `unwrap_or_else` + exit?
   - **Recommendation**: `expect("failed to open mempool database")` for consistency with existing code (line 73, 122, 134).

3. **Rollback strategy**: If persistent mempool causes issues in production, how to revert?
   - **Answer**: Change one line in `main.rs` back to `InMemoryTxPool::new()`, remove `mempool` dependency from `Cargo.toml`. Persistent mempool DB ignored after rollback (no migration needed, mempool data is transient by nature).

4. **Logging on recovery**: Log recovered transaction count on startup?
   - **Recommendation**: Add log after `PersistentTxPool::open()` if recovery count is exposed via API (e.g., `open()` returns `(Self, usize)` with count). Out of scope for initial implementation — add in observability phase.
