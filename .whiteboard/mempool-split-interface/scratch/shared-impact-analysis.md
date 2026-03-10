# Shared Impact Analysis — mempool-split-interface

## Summary

| Symbol | Call Sites | Files | Crates |
|---|---|---|---|
| `MempoolStore` (struct) | 15+ | 3 (store.rs, persistent.rs, lib.rs) + tests | mempool |
| `PersistentTxPool` | 12+ | 3 (persistent.rs, lib.rs, integration.rs) + whirlpool-node | mempool, whirlpool-node |
| `MempoolError` | 10+ | 3 (error.rs, store.rs, persistent.rs) + lib.rs | mempool |
| `TxSource` impl | 1 impl block | persistent.rs | mempool |

## Per-Symbol Analysis

### MempoolStore (struct → trait + impl)

#### Direct Calls
- `crates/mempool/src/store.rs`: Definition. Methods: `open()`, `push()`, `drain_pending()`, `load_next_key()`.
- `crates/mempool/src/persistent.rs`: `MempoolStore::open(path)` in `PersistentTxPool::open()`.
- `crates/mempool/src/store.rs` (tests): 7 tests call `MempoolStore::open()`, `.push()`, `.drain_pending()`.

#### Imports
- `crates/mempool/src/persistent.rs`: `use crate::MempoolStore;`
- `crates/mempool/src/store.rs` (tests): implicit (same module)

#### Type Annotations
- `crates/mempool/src/persistent.rs`: `store: MempoolStore` field in `PersistentTxPool`
- `crates/mempool/src/store.rs` (tests): `let store = MempoolStore::open(...)` locals

#### Re-exports
- `crates/mempool/src/lib.rs`: `pub use store::MempoolStore;`

#### Trait Implementations
- None currently. Will BECOME a trait.

#### External Usage
- **NOT used directly by whirlpool-node** (only PersistentTxPool is imported)

### PersistentTxPool

#### Direct Calls
- `crates/mempool/src/persistent.rs`: Definition. `PersistentTxPool::open()`.
- `crates/mempool/tests/integration.rs`: 6 tests call `PersistentTxPool::open()`.
- `crates/whirlpool-node/src/main.rs`: `PersistentTxPool::open(...)`, cast to `Arc<dyn TxSource>`.

#### Imports
- `crates/mempool/src/lib.rs`: `pub use persistent::PersistentTxPool;`
- `crates/mempool/tests/integration.rs`: `use mempool::PersistentTxPool;`
- `crates/whirlpool-node/src/main.rs`: `use mempool::PersistentTxPool;`

#### Type Annotations
- `crates/whirlpool-node/src/main.rs`: used as `Arc<dyn TxSource>` after construction
- `crates/mempool/tests/integration.rs`: `Arc<dyn TxSource>` coercion tests

### MempoolError

#### Direct Calls
- `crates/mempool/src/error.rs`: Definition.
- `crates/mempool/src/store.rs`: Return type in `open()`, `push()`, `drain_pending()`, `load_next_key()`.
- `crates/mempool/src/persistent.rs`: Return type in `open()`.

#### Imports
- `crates/mempool/src/store.rs`: `use crate::error::MempoolError;`
- `crates/mempool/src/persistent.rs`: `use crate::MempoolError;`
- `crates/mempool/src/lib.rs`: `pub use error::MempoolError;`

#### External Usage
- **NOT used directly by whirlpool-node** (errors handled by PersistentTxPool internally)

### TxSource impl for PersistentTxPool

#### Location
- `crates/mempool/src/persistent.rs`: `impl TxSource for PersistentTxPool`

#### Dependencies
- `app::traits::TxSource` (trait definition)
- Delegates to `self.store.push()` and `self.store.drain_pending()`

#### External Impact
- `crates/whirlpool-node/src/main.rs`: relies on this impl for `Arc<dyn TxSource>` cast
- `crates/mempool/tests/integration.rs`: all 6 tests exercise through TxSource trait object
