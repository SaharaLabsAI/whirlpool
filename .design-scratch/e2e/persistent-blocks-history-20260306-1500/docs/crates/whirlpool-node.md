# whirlpool-node crate — Persistent Block Storage Contract

## Purpose

**Today**: Binary entry point for the Whirlpool consensus node. Constructs `RethStateDb`, `EvmApplication`, `ApplicationAdapter`, `CommonwareEngine`, `FinalizationSink`, `EthRpcContext`, and starts the consensus engine + RPC server. The `state_db` (`Arc<RwLock<RethStateDb>>`) is shared between `EvmApplication` (for state reads/writes) and `EthRpcContext` (for RPC state queries).

**Changes**: Wire `BlockStorage` into the RPC context so `eth_getBlock*` endpoints can query persistent block data. The `RethStateDb` instance already implements both `StateDb` and `BlockStorage`, so the same `Arc<RwLock<RethStateDb>>` is passed to both `EvmApplication` and `EthRpcContext`. Additionally, integrate the finalization persistence hook so blocks are stored on finalization.

## Public API Changes

None. This is a binary crate — no public API. Changes are to internal wiring in `src/main.rs`.

### Modified file: `src/main.rs`

**RPC context construction** — pass `block_storage` parameter:

```rust
// BEFORE (current):
let rpc_ctx = rpc::context::EthRpcContext::new(tx_pool, state_db, SAHARA_CHAIN_ID);

// AFTER:
let rpc_ctx = rpc::context::EthRpcContext::new(
    tx_pool,
    state_db.clone(),       // StateDb
    state_db.clone(),       // BlockStorage (same RethStateDb instance)
    SAHARA_CHAIN_ID,
);
```

**Finalization persistence hook** — add block persistence on finalization events:

The current flow creates a `FinalizationSink` that only tracks height. To add block persistence, create a composite sink or extend the finalization handling:

```rust
use consensus::event::{ConsensusEvent, EventSink};

/// Composite finalization handler that:
/// 1. Persists the finalized block via BlockStorage
/// 2. Delegates to FinalizationSink for height tracking
struct PersistingFinalizationSink {
    evm_app: EvmApplication<RethStateDb>,
    inner_sink: Arc<FinalizationSink<EvmBlock>>,
}

impl EventSink for PersistingFinalizationSink {
    type Block = EvmBlock;

    async fn handle(&self, event: ConsensusEvent<EvmBlock>) {
        match &event {
            ConsensusEvent::Finalized { block, .. } => {
                // Persist block + receipts to MDBX
                if let Err(e) = self.evm_app.store_finalized_block(block) {
                    tracing::error!(?e, "failed to persist finalized block");
                }
            }
            _ => {}
        }
        // Delegate to inner sink for height tracking + logging
        self.inner_sink.handle(event).await;
    }
}
```

**Complete wiring change** in `main()`:

```rust
// Current:
let height = Arc::new(AtomicU64::new(0));
let sink = Arc::new(FinalizationSink::new(Arc::clone(&height)));
// ...
let evm_app = EvmApplication::new(evm_config, state_db.clone(), tx_pool.clone());
let app = Arc::new(ApplicationAdapter::new(evm_app));
let engine = CommonwareEngine::new(app, sink, engine_config, network_provider, context.clone());

// After:
let height = Arc::new(AtomicU64::new(0));
let inner_sink = Arc::new(FinalizationSink::new(Arc::clone(&height)));
let evm_app = EvmApplication::new(evm_config, state_db.clone(), tx_pool.clone());
let sink = Arc::new(PersistingFinalizationSink {
    evm_app: evm_app.clone(),
    inner_sink,
});
let app = Arc::new(ApplicationAdapter::new(evm_app));
let engine = CommonwareEngine::new(app, sink, engine_config, network_provider, context.clone());
```

## Internal Changes

### PersistingFinalizationSink

New struct defined locally in `main.rs` (or extracted to a module if complexity grows). Wraps `EvmApplication` and `FinalizationSink`, implementing `EventSink` to intercept `ConsensusEvent::Finalized` events and call `store_finalized_block()` before delegating to the inner sink.

### No new initialization

`RethStateDb` is constructed exactly as before via `state_reth::open_state_db(&db_path)`. The `init_db()` call inside `open_state_db` already creates all MDBX tables including the block storage tables (`Headers`, `Transactions`, `Receipts`, etc.).

## Dependencies

### No new dependencies

All required crates already exist in `Cargo.toml`:

- `app-evm = { path = "../app-evm" }` — already present
- `rpc-eth = { path = "../rpc-eth" }` — already present
- `state-reth = { path = "../state-reth" }` — already present
- `consensus = { path = "../consensus" }` — already present (for `EventSink`, `ConsensusEvent`)
- `consensus-simplex = { path = "../consensus-simplex" }` — already present (for `FinalizationSink`)

## Error Types

No new error types. Block persistence errors are logged via `tracing::error!()` and do not halt the node. This is a deliberate choice — finalization must not be blocked by storage failures.

## Test Surface

### No new unit tests in this crate

`whirlpool-node` is a binary crate with minimal logic. The persistence hook correctness is validated by:

1. **app-evm** tests: `store_finalized_block()` unit tests
2. **state-reth** tests: `BlockStorage` round-trip tests
3. **rpc-eth** tests: `get_block_by_number/hash` endpoint tests
4. **Integration tests** (in `testing/integration-tests`): End-to-end test: start node → propose → finalize → RPC query returns block

### Manual verification

- Start node, wait for blocks to finalize, call `eth_getBlockByNumber("latest", true)` via curl
- Verify response contains correct block data

## Integration Points

| Connected Crate | Direction | Interface | Wiring |
|-----------------|-----------|-----------|--------|
| `state-reth` | Creates | `RethStateDb` (impls `StateDb` + `BlockStorage`) | `open_state_db(&db_path)` |
| `app-evm` | Creates | `EvmApplication<RethStateDb>` | `EvmApplication::new(evm_config, state_db.clone(), tx_pool.clone())` |
| `rpc-eth` | Creates | `EthRpcContext<RethStateDb, RethStateDb>` | `EthRpcContext::new(tx_pool, state_db.clone(), state_db.clone(), chain_id)` |
| `consensus-simplex` | Creates | `FinalizationSink<EvmBlock>` | Wrapped inside `PersistingFinalizationSink` |
| `consensus-simplex` | Creates | `CommonwareEngine` | Receives `PersistingFinalizationSink` as sink |

**Key architectural point**: `state_db: Arc<RwLock<RethStateDb>>` is shared across three consumers:
1. `EvmApplication` — state reads/writes during propose/verify + block persistence on finalization
2. `EthRpcContext` (as `state_db`) — state queries for `get_balance`, `get_transaction_count`
3. `EthRpcContext` (as `block_storage`) — block queries for `eth_getBlockByNumber`, `eth_getBlockByHash`

All access is synchronized via the `RwLock`. Finalization writes (`store_block`) and RPC reads (`get_block_by_number`) are serialized by the lock.

**Source**: STRATEGY.md Crate Changes whirlpool-node section, CRATES.md whirlpool-node section, DOMAINS.md Integration Point 3, WORKSPACE.md Integration Point 4
