# STRATEGY — Mempool Interface/Implementation Split

## Approach: Scaffolding (Incremental Move)

Create the new crate first, move code into it step-by-step, then strip the old crate to interface-only. Every step compiles.

**Why scaffolding over in-place transform:** The mempool crate is small (4 files, ~250 lines). Moving code to a new crate in small steps is straightforward and each step independently compiles. No need for temporary compatibility shims.

## Crate-Level Design

### `mempool` (interface, modified)

```
crates/mempool/
├── Cargo.toml          (deps: thiserror only)
├── src/
│   ├── lib.rs          (pub mod error, pub mod traits; pub use)
│   ├── error.rs        (MempoolError with Storage/Io variants)
│   └── traits.rs       (MempoolStore trait: push, drain_pending)
```

**Public API:**
- `mempool::MempoolStore` — trait
- `mempool::MempoolError` — error enum

### `mempool-mdbx` (implementation, new)

```
crates/mempool-mdbx/
├── Cargo.toml          (deps: mempool, app, reth-libmdbx; dev: tempfile)
├── src/
│   ├── lib.rs          (pub mod store, pub mod persistent; pub use)
│   ├── store.rs        (MdbxMempoolStore struct, impl MempoolStore)
│   └── persistent.rs   (PersistentTxPool struct, impl TxSource)
├── tests/
│   └── integration.rs  (moved from mempool/tests/)
```

**Public API:**
- `mempool_mdbx::MdbxMempoolStore` — concrete MDBX store
- `mempool_mdbx::PersistentTxPool` — TxSource adapter

## Error Strategy

`MempoolError` lives in `mempool` (interface). Variants:
- `Storage(String)` — any storage backend error (renamed from `Mdbx`)
- `Io(std::io::Error)` — filesystem errors

The `From<reth_libmdbx::Error>` impl lives in `mempool-mdbx`:
```rust
impl From<reth_libmdbx::Error> for MempoolError {
    fn from(e: reth_libmdbx::Error) -> Self {
        MempoolError::Storage(e.to_string())
    }
}
```

This is valid because `MempoolError` is a foreign type to `mempool-mdbx` BUT the `From` impl uses a foreign trait (`From`) with a foreign type (`reth_libmdbx::Error`) — **orphan rule violation**. 

**Resolution:** Keep `From<reth_libmdbx::Error>` as a helper function in `mempool-mdbx` instead, or use a newtype. Actually, since `MempoolError::Storage(String)` just takes a String, `mempool-mdbx` can do `MempoolError::Storage(e.to_string())` directly without a `From` impl. Simpler and avoids orphan issues.

## Naming Conventions

| Current | After |
|---|---|
| `MempoolStore` (struct) | `MempoolStore` (trait) + `MdbxMempoolStore` (struct) |
| `MempoolError::Mdbx(String)` | `MempoolError::Storage(String)` |
| `PersistentTxPool` | `PersistentTxPool` (unchanged name, new crate) |

## Key Decisions

1. **No generics on PersistentTxPool** — concrete `MdbxMempoolStore` field. KISS.
2. **No `From` orphan impls** — direct `MempoolError::Storage(e.to_string())` construction in mempool-mdbx.
3. **`traits.rs` in interface crate** — this IS the interface crate (not a mixed crate), so `traits.rs` is correct per conventions.
4. **`mempool` drops `app` dep** — the MempoolStore trait doesn't reference TxSource. Only mempool-mdbx needs `app` for the TxSource impl.
