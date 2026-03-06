# app-evm crate — Persistent Block Storage Contract

## Purpose

**Today**: Implements EVM execution via reth-evm. `EvmApplication<DB>` implements the `Application` trait with `genesis()`, `propose()`, and `verify()` methods. During `propose()`, transactions are executed, receipts are computed for the `receipts_root`, and then discarded. Conversion functions `build_header_from_evm_block()` and `decode_transactions()` exist in `executor.rs` for internal use.

**Changes**:
1. Make `build_header_from_evm_block()` public so `state-reth` can use it for EvmBlock-to-Header conversion
2. Add a `receipts` field to `EvmApplication<DB>` to retain receipts computed during `propose()` for later persistence on finalization
3. Add a finalization persistence hook that calls `BlockStorage::store_block()` when a block is finalized

## Public API Changes

### Modified file: `src/executor.rs`

**Visibility change** — `build_header_from_evm_block` becomes public:

```rust
// BEFORE (current):
fn build_header_from_evm_block(block: &EvmBlock) -> Header { ... }

// AFTER:
/// Converts an `EvmBlock` into a reth `Header`.
///
/// Maps EvmBlock fields to Header fields. Sets gas_limit to 30_000_000
/// and difficulty to U256::ZERO. Used by state-reth BlockStorage for
/// MDBX persistence.
pub fn build_header_from_evm_block(block: &EvmBlock) -> Header {
    Header {
        number: block.height,
        parent_hash: B256::from(block.parent_id),
        state_root: B256::from(block.state_root),
        transactions_root: B256::from(block.transactions_root),
        receipts_root: B256::from(block.receipts_root),
        gas_limit: 30_000_000,
        gas_used: block.gas_used,
        timestamp: block.timestamp,
        difficulty: U256::ZERO,
        extra_data: Bytes::default(),
        ..Header::default()
    }
}
```

**`decode_transactions` is already public** — no change needed:

```rust
// Already public (no change):
pub fn decode_transactions(raw_txs: &[Vec<u8>]) -> Result<Vec<RecoveredTx>, EvmAppError> { ... }
```

### Modified file: `src/lib.rs`

Add re-export of the now-public conversion function:

```rust
pub mod config;
pub mod error;
pub mod executor;
pub mod traits;

pub use config::{build_sahara_chain_spec, WhirlpoolEvmConfig, SAHARA_CHAIN_ID};
pub use error::EvmAppError;
pub use executor::build_header_from_evm_block;  // NEW re-export
```

### Modified struct: `EvmApplication<DB>` in `src/executor.rs`

```rust
use std::sync::Mutex;
use alloy_consensus::Receipt;
use state::BlockStorage;

#[derive(Clone)]
pub struct EvmApplication<DB> {
    evm_config: WhirlpoolEvmConfig,
    state_db: Arc<RwLock<DB>>,
    tx_source: Arc<dyn TxSource + Send + Sync>,
    /// Receipts from the most recent `propose()` call, retained for
    /// persistence during finalization. Wrapped in Mutex for interior
    /// mutability (propose stores, finalization handler consumes).
    pending_receipts: Arc<Mutex<Option<Vec<Receipt>>>>,
}
```

**Constructor change**:

```rust
impl<DB> EvmApplication<DB> {
    pub fn new(
        evm_config: WhirlpoolEvmConfig,
        state_db: Arc<RwLock<DB>>,
        tx_source: Arc<dyn TxSource + Send + Sync>,
    ) -> Self {
        Self {
            evm_config,
            state_db,
            tx_source,
            pending_receipts: Arc::new(Mutex::new(None)),  // NEW
        }
    }
}
```

### New method: `store_finalized_block`

```rust
impl<DB> EvmApplication<DB>
where
    DB: BlockStorage + Send + Sync + 'static,
{
    /// Persist a finalized block and its receipts to the block storage backend.
    ///
    /// Called from the finalization event handler. Retrieves receipts stored
    /// during the preceding `propose()` call and writes the block + receipts
    /// to persistent storage via `BlockStorage::store_block()`.
    pub fn store_finalized_block(&self, block: &EvmBlock) -> Result<(), EvmAppError> {
        let receipts = self
            .pending_receipts
            .lock()
            .expect("pending_receipts lock poisoned")
            .take()
            .unwrap_or_default();

        let mut db = self.state_db.write().expect("state_db lock poisoned");
        db.store_block(block, &receipts)
            .map_err(|e| EvmAppError::State(e.to_string()))
    }
}
```

## Internal Changes

### Receipt retention in `propose()`

In the `propose()` method, after computing `receipts_root` (line ~203-206 in current code), store the receipts before they are dropped:

```rust
// In propose(), after computing receipts_root:
let receipts_root =
    ordered_trie_root_with_encoder(&execution_result.receipts, |receipt, out| {
        receipt.with_bloom_ref().encode_2718(out);
    });

// NEW: Store receipts for later persistence on finalization
{
    let mut pending = self.pending_receipts.lock().expect("lock poisoned");
    *pending = Some(execution_result.receipts.clone());
}
```

### Finalization handler integration

The caller (consensus event handler in `whirlpool-node` or `ApplicationAdapter`) calls `store_finalized_block()` when receiving `ConsensusEvent::Finalized`. This is wired externally — `app-evm` provides the method but does not directly implement `EventSink`.

> **Design note**: The current architecture has `FinalizationSink` in `consensus-simplex` handling finalization events. The persistence call will be integrated at the node wiring level where `ConsensusEvent::Finalized` is received — see `whirlpool-node.md`.

## Dependencies

### No new dependencies

All required crates already exist in `Cargo.toml`:

- `alloy-consensus = "1.4.3"` — already present (used for `TxReceipt`)
- `state = { path = "../state" }` — already present
- `state-reth = { path = "../state-reth" }` — already present
- `app = { path = "../app" }` — already present

### Existing dependency now used for new trait

- `state` — now also imports `BlockStorage` trait (previously only `StateDb` was used)

## Error Types

### No new error variants

The existing `EvmAppError::State(String)` variant covers `BlockStorage` errors:

```rust
// Existing (no change):
pub enum EvmAppError {
    Execution(String),
    StateRootMismatch { expected: [u8; 32], computed: [u8; 32] },
    State(String),          // Used for BlockStorage errors
    InvalidBlock(String),
    InvalidTransaction(String),
}
```

The `From<state_reth::RethStateError>` impl already converts to `EvmAppError::State`:

```rust
// Existing (no change):
impl From<state_reth::RethStateError> for EvmAppError {
    fn from(err: state_reth::RethStateError) -> Self {
        EvmAppError::State(err.to_string())
    }
}
```

## Test Surface

### Unit tests

1. **Receipts retained after propose** — After `propose()`, verify `pending_receipts` contains the receipts (non-None, correct count)
2. **Receipts cleared after store_finalized_block** — After calling `store_finalized_block()`, verify `pending_receipts` is `None`
3. **store_finalized_block with no pending receipts** — Verify it calls `store_block` with empty receipts slice (no panic)
4. **build_header_from_evm_block public access** — Verify the function can be called from outside the module (compile test)

### Integration tests

5. **Propose → store → retrieve round-trip** — `propose()` a block, call `store_finalized_block()`, then query `state_db.get_block_by_number()` and verify the retrieved block matches
6. **Multiple blocks sequential** — Propose and finalize blocks 1, 2, 3 sequentially, verify all are retrievable

## Integration Points

| Connected Crate | Direction | Interface | Data Flow |
|-----------------|-----------|-----------|-----------|
| `state` | Depends on | `BlockStorage` trait | `DB: BlockStorage` bound on `store_finalized_block` |
| `state-reth` | Depends on (impl) | `RethStateDb` implements `BlockStorage` | `store_block()` writes to MDBX |
| `state-reth` | Consumed by | `build_header_from_evm_block()` (now public) | state-reth imports this for EvmBlock -> Header conversion |
| `whirlpool-node` | Called by | `store_finalized_block()` | Node wiring calls this on finalization events |
| `app` | Depends on | `EvmBlock`, `ExecutionResult` types | Block/result types used in Application trait |

**Key data flow**: `propose()` stores `execution_result.receipts` in `pending_receipts` -> finalization event triggers `store_finalized_block()` -> `state_db.store_block(&block, &receipts)` -> MDBX persistence

**Source**: STRATEGY.md Stream 2, CRATES.md app-evm section, DOMAINS.md Integration Point 1
