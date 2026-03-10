# CHANGES — mempool (interface crate)

## Current State

The `mempool` crate is a monolithic crate containing both interface and MDBX implementation:
- `store.rs` — `MempoolStore` struct (MDBX-backed storage)
- `persistent.rs` — `PersistentTxPool` struct (TxSource adapter)
- `error.rs` — `MempoolError` enum
- Dependencies: `app`, `reth-libmdbx`

## Proposed Changes

1. **Add `traits.rs`** — New `MempoolStore` trait with `push()` and `drain_pending()` methods.
2. **Remove `store.rs`** — Struct moves to `mempool-mdbx`.
3. **Remove `persistent.rs`** — Struct moves to `mempool-mdbx`.
4. **Rename error variant** — `MempoolError::Mdbx(String)` → `MempoolError::Storage(String)`.
5. **Remove `From<reth_libmdbx::Error>`** — Orphan-safe: impl moves to `mempool-mdbx` as direct construction.
6. **Strip dependencies** — Remove `app`, `reth-libmdbx`, `tempfile`. Add `thiserror` if not already present.
7. **Update `lib.rs`** — Export only `error`, `traits`, `MempoolError`, `MempoolStore` (trait).

## Impact on Dependents

| Dependent | Impact | Migration |
|---|---|---|
| `mempool-mdbx` (new) | Depends on this for trait + error | — |
| `whirlpool-node` | Drops direct dep on `mempool`, deps on `mempool-mdbx` instead | Update Cargo.toml + import |

## Migration Notes

- During migration, the crate temporarily exports both the old struct (`MempoolStore`) and the new trait (as `MempoolStoreTrait`). After Step 7, only the trait remains as `MempoolStore`.
- The `From<io::Error>` impl for `MempoolError` stays in this crate (generic, no orphan issue).
