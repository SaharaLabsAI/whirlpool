# IMPACT — Mempool Interface/Implementation Split

## Blast Radius

| Scope | Count | Risk |
|---|---|---|
| Crates modified | 2 (mempool, whirlpool-node) | Low |
| Crates created | 1 (mempool-mdbx) | Low |
| Symbols moved/changed | 4 | Medium |
| Tests affected | 16 (all move) | Low |
| External consumers affected | 1 (whirlpool-node) | Low |

**Overall Risk: LOW-MEDIUM.** Small blast radius, single external consumer, well-understood patterns.

## Call Site Analysis

### MempoolStore (struct → trait + renamed impl)

| Old Path | New Path | Change Type | Sites | Files | Migration | Risk |
|---|---|---|---|---|---|---|
| `mempool::store::MempoolStore` (struct) | `mempool::MempoolStore` (trait) | TYPE_CHANGE | — | `mempool/src/lib.rs` | Define trait matching struct API | Medium |
| `mempool::store::MempoolStore` (struct) | `mempool_mdbx::MdbxMempoolStore` (struct) | MOVE+RENAME | 15+ | store.rs, persistent.rs, 7 tests | Move file, rename struct, impl trait | Medium |
| `MempoolStore::open(path)` | `MdbxMempoolStore::open(path)` | RENAME | 8+ | persistent.rs, 7 unit tests | Search-replace | Low |
| `store.push(tx)` / `store.drain_pending()` | Same method names | NO_CHANGE | 5+ | store.rs, persistent.rs | None (methods match trait) | Low |

### PersistentTxPool (move crate)

| Old Path | New Path | Change Type | Sites | Files | Migration | Risk |
|---|---|---|---|---|---|---|
| `mempool::PersistentTxPool` | `mempool_mdbx::PersistentTxPool` | MOVE | 1 | whirlpool-node/main.rs | Update import | Low |
| `mempool::PersistentTxPool` | `mempool_mdbx::PersistentTxPool` | MOVE | 6 | integration.rs | Update import | Low |
| `mempool::persistent::PersistentTxPool` (def) | `mempool_mdbx::persistent::PersistentTxPool` | MOVE | 1 | persistent.rs | Move file | Low |

### MempoolError (stays, variant rename)

| Old Path | New Path | Change Type | Sites | Files | Migration | Risk |
|---|---|---|---|---|---|---|
| `mempool::MempoolError` | `mempool::MempoolError` | STAYS | — | error.rs | None | Low |
| `MempoolError::Mdbx(String)` | `MempoolError::Storage(String)` | RENAME_VARIANT | 2 | error.rs, store.rs (Display impl) | Search-replace | Low |
| `From<reth_libmdbx::Error>` | Moves to `mempool-mdbx` | MOVE_IMPL | 1 | error.rs → mempool-mdbx | Move From impl | Medium |

### TxSource impl (moves with PersistentTxPool)

| Old Path | New Path | Change Type | Sites | Files | Migration | Risk |
|---|---|---|---|---|---|---|
| `impl TxSource for PersistentTxPool` | Same (in mempool-mdbx) | MOVE | 1 | persistent.rs | Moves with file | Low |

## Trait Impact

### New Trait: `mempool::MempoolStore`

```rust
pub trait MempoolStore: Send + Sync {
    fn push(&self, tx: Vec<u8>) -> Result<(), MempoolError>;
    fn drain_pending(&self) -> Result<Vec<Vec<u8>>, MempoolError>;
}
```

**Note**: `open()` is NOT part of the trait — it's a constructor specific to each implementation. The trait only defines the operational interface. This matches how `StateDb` works (no `new` in trait).

### Existing Trait: `app::traits::TxSource`
- **Unchanged.** `PersistentTxPool` continues to impl `TxSource`, just from a different crate.

## Dependency Graph Impact

### Before
```
whirlpool-node → mempool → app, reth-libmdbx
```

### After
```
whirlpool-node → mempool-mdbx → mempool → app
                              → reth-libmdbx
```

- `mempool` loses `reth-libmdbx` dependency ✅
- `mempool-mdbx` gains `mempool` dependency (for trait + error) ✅
- `whirlpool-node` switches `mempool` → `mempool-mdbx` ✅
- No circular dependencies ✅

## Key Decisions

1. **MempoolStore trait excludes `open()`** — constructors are impl-specific, not trait methods.
2. **PersistentTxPool stays concrete** (holds `MdbxMempoolStore` directly, not generic over `S: MempoolStore`). Simpler. If generics needed later, it's additive.
3. **MempoolError::Mdbx → MempoolError::Storage** — rename for storage-agnosticism. The `From<reth_libmdbx::Error>` impl moves to `mempool-mdbx` crate. `From<io::Error>` stays in interface.
4. **`mempool` interface crate has NO dep on `reth-libmdbx`** — only deps on `thiserror` (or std Error) for MempoolError.
