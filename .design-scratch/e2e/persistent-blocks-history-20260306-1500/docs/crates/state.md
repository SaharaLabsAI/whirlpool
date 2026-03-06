# state crate — Persistent Block Storage Contract

## Purpose

**Today**: Defines the `StateDb` trait (11 methods) and `StateError` enum for abstract state management. Provides the shared interface between `state-reth` (MDBX), `state-memory` (in-memory), and consumers (`app-evm`, `rpc-eth`).

**Changes**: Add a new `BlockStorage` trait in a new module (`block_storage.rs`) that defines the contract for persisting and querying finalized blocks with receipts. This is an additive change — no existing types or traits are modified.

## Public API Changes

### New file: `src/block_storage.rs`

```rust
use app::EvmBlock;
use alloy_consensus::Receipt;

/// Trait for persistent block and receipt storage.
///
/// Implementations store finalized blocks (as EvmBlock + receipts) and
/// support retrieval by block number or block hash.
pub trait BlockStorage: Send + Sync {
    /// Persist a finalized block and its transaction receipts.
    ///
    /// Implementations must atomically store the block header, transactions,
    /// transaction indices, and per-transaction receipts.
    fn store_block(&mut self, block: &EvmBlock, receipts: &[Receipt]) -> Result<(), Self::Error>;

    /// Retrieve a block by its height. Returns `None` if the block has not
    /// been persisted.
    fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>, Self::Error>;

    /// Retrieve a block by its header hash. Returns `None` if no block with
    /// this hash has been persisted.
    fn get_block_by_hash(&self, hash: &[u8; 32]) -> Result<Option<EvmBlock>, Self::Error>;

    /// Retrieve all transaction receipts for a given block number.
    /// Returns an empty `Vec` if the block does not exist or has no receipts.
    fn get_receipts_by_block(&self, number: u64) -> Result<Vec<Receipt>, Self::Error>;

    /// The error type for storage operations.
    type Error: std::error::Error + Send + Sync + 'static;
}
```

> **Design note**: The trait uses an associated `Error` type following the same pattern as the existing `StateDb` trait. The STRATEGY.md shows `Result<()>` but the concrete signature uses an associated error to match crate conventions.

### Modified file: `src/lib.rs`

```rust
pub mod block_storage;  // NEW
pub mod error;
pub mod traits;

// Re-export public types for convenience
pub use alloy_genesis::GenesisAccount;
pub use block_storage::BlockStorage;  // NEW
pub use error::StateError;
pub use traits::StateDb;
```

### Unchanged

- `StateDb` trait — no modifications
- `StateError` enum — no modifications
- `GenesisAccount` re-export — no modifications

## Internal Changes

None. This crate only defines the trait; implementation lives in `state-reth`.

## Dependencies

### New in `Cargo.toml`

```toml
[dependencies]
# ... existing ...
app = { path = "../app" }            # For EvmBlock type in BlockStorage trait signature
alloy-consensus = "1.4.3"           # For Receipt type in BlockStorage trait signature
```

### Existing (unchanged)

- `revm = "34"`
- `thiserror = "2"`
- `alloy-genesis = "1.5"`

## Error Types

No new error types in this crate. The `BlockStorage` trait uses an associated `type Error` — each implementation defines its own error. The existing `StateError` enum is not modified.

## Test Surface

- **Compile-time**: Verify `BlockStorage` trait is object-safe (can be used as `Arc<dyn BlockStorage<Error = ...>>`)
- **Trait bound checks**: Ensure `Send + Sync` bounds compile with async contexts
- **No runtime tests needed** in `state` crate itself — the trait has no default implementations

## Integration Points

| Consumer Crate | Usage | Interface |
|----------------|-------|-----------|
| `state-reth` | Implements `BlockStorage` for `RethStateDb` | `impl BlockStorage for RethStateDb` |
| `app-evm` | Calls `store_block()` during finalization | `&mut dyn BlockStorage` (via `RethStateDb`) |
| `rpc-eth` | Calls `get_block_by_number()`, `get_block_by_hash()`, `get_receipts_by_block()` | `Arc<dyn BlockStorage>` field in `EthRpcContext` |
| `whirlpool-node` | Wires `RethStateDb` (which impls both `StateDb` and `BlockStorage`) into RPC context | Constructor parameter |

**Source**: STRATEGY.md Stream 1, CRATES.md state section, DOMAINS.md Storage Domain
