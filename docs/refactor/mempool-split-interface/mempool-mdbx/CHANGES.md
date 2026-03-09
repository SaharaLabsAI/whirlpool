# CHANGES — mempool-mdbx (new implementation crate)

## Current State

Does not exist. All MDBX implementation currently lives in `crates/mempool/`.

## Proposed Changes

1. **Create crate** at `crates/mempool-mdbx/` with deps: `mempool` (path), `app` (path), `reth-libmdbx` (vendor path). Dev: `tempfile`.
2. **`store.rs`** — `MdbxMempoolStore` struct (renamed from `MempoolStore`). Contains:
   - `open(path: &Path) -> Result<Self, MempoolError>` — constructor
   - `impl MempoolStore for MdbxMempoolStore` — trait implementation
   - Private `load_next_key()` helper
   - Helper fn for `reth_libmdbx::Error` → `MempoolError::Storage(String)` conversion
   - 7 unit tests (moved from mempool/src/store.rs)
3. **`persistent.rs`** — `PersistentTxPool` struct. Contains:
   - `open(path: &Path) -> Result<Self, MempoolError>` — constructor
   - `impl TxSource for PersistentTxPool` — trait implementation
   - 3 unit tests (moved from mempool/src/persistent.rs)
4. **`tests/integration.rs`** — 6 integration tests (moved from mempool/tests/)
5. **`lib.rs`** — Re-exports: `MdbxMempoolStore`, `PersistentTxPool`

## Impact on Dependents

| Dependent | Impact | Migration |
|---|---|---|
| `whirlpool-node` | New dep source for `PersistentTxPool` | Update Cargo.toml + import path |

## Migration Notes

- `MdbxMempoolStore` implements `mempool::MempoolStore` trait — verified by TN-001.
- Error conversion: `reth_libmdbx::Error` → `MempoolError::Storage(e.to_string())` — no orphan `From` impl needed.
- All tests from mempool carry over 1:1 with import path updates only.
