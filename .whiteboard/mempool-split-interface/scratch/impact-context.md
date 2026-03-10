# Impact Context — mempool-split-interface

## Blast Radius

| Symbol | Change | Sites Affected | Risk |
|---|---|---|---|
| `MempoolStore` (struct→trait) | Extract trait, move impl | 3 source files + 7 unit tests | Medium — method signatures must match exactly |
| `PersistentTxPool` | Move crate | persistent.rs + 9 tests + whirlpool-node | Low — import path change only |
| `MempoolError` | Stays, possibly rename variant | error.rs + store.rs + persistent.rs | **Medium — Mdbx variant naming decision** |
| `TxSource` impl | Moves with PersistentTxPool | persistent.rs | Low — no API change |

## Critical Call Site Analysis

### whirlpool-node (only external consumer)
- **Current**: `use mempool::PersistentTxPool` → `PersistentTxPool::open(path)` → `Arc<dyn TxSource>`
- **After**: `use mempool_mdbx::PersistentTxPool` — same API, different import path
- **Risk**: Minimal. Single import line change.

### Internal persistent.rs → store.rs coupling
- **Current**: `PersistentTxPool` holds `MempoolStore` struct as field, calls `.push()` and `.drain_pending()`
- **After**: `PersistentTxPool` holds `MdbxMempoolStore` (or generic `impl MempoolStore`). If generic, TxSource impl becomes `impl<S: MempoolStore> TxSource for PersistentTxPool<S>` — more complex.
- **Decision**: Keep PersistentTxPool concrete (holds MdbxMempoolStore directly). Simpler. Follows state-memory pattern.

## BLOCKER: BLK-001 — MempoolError::Mdbx Variant
- `MempoolError::Mdbx(String)` in interface crate is storage-specific naming.
- **Recommended**: Rename to `MempoolError::Storage(String)` in interface. Both `From<reth_libmdbx::Error>` and `From<io::Error>` impls move to `mempool-mdbx` or stay split.
- **Alternative**: Keep as-is. It's already a String (no type dependency on mdbx). Pragmatic but leaky naming.
