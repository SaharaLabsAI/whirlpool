# Task 04: Move PersistentTxPool to mempool-mdbx

| Field | Value |
|---|---|
| Status | `completed` |
| Dependencies | `03-move-store-impl` |
| Wave | 4 |
| Complexity | M (4 files) |
| Target Crate(s) | `mempool-mdbx` |
| Migration Step | Step 4 |
| Change Type | MOVE |

## Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

Tasks 01–03 must be completed. `MdbxMempoolStore` must exist in `mempool-mdbx`.

## Context

### Before
`PersistentTxPool` lives in `crates/mempool/src/persistent.rs`, holds a `MempoolStore` struct, implements `TxSource`.

### After
`PersistentTxPool` in `crates/mempool-mdbx/src/persistent.rs`, holds `MdbxMempoolStore`, implements `TxSource`. 3 unit tests moved.

## What to Do

### Phase 1: Tests

Move TB-008 through TB-010 from `crates/mempool/src/persistent.rs` to `crates/mempool-mdbx/src/persistent.rs`. Update:
- `use crate::{MempoolError, MempoolStore}` → `use mempool::MempoolError; use crate::MdbxMempoolStore;`
- `PersistentTxPool::open(` stays the same (struct moves with it)
- Any `use app::traits::TxSource` stays the same

### Phase 2: Implement

1. **Update `crates/mempool-mdbx/Cargo.toml`** — Add `app` dependency:
   ```toml
   app = { path = "../app" }
   ```

2. **Create `crates/mempool-mdbx/src/persistent.rs`** — Copy from `crates/mempool/src/persistent.rs`:
   - Update `use` statements:
     - `use crate::{MempoolError, MempoolStore}` → `use mempool::MempoolError; use crate::MdbxMempoolStore;`
   - Update struct field: `store: MempoolStore` → `store: MdbxMempoolStore`
   - Update `open()`: `MempoolStore::open(path)` → `MdbxMempoolStore::open(path)`
   - `impl TxSource for PersistentTxPool` stays the same (delegates to store methods)
   - Include moved test module

3. **Update `crates/mempool-mdbx/src/lib.rs`**:
   ```rust
   pub mod persistent;
   pub mod store;

   pub use persistent::PersistentTxPool;
   pub use store::MdbxMempoolStore;
   ```

### Phase 3: Consumers
No consumer changes yet.

## Rollback

```bash
rm crates/mempool-mdbx/src/persistent.rs
git checkout crates/mempool-mdbx/src/lib.rs crates/mempool-mdbx/Cargo.toml
```

## Must NOT Do

- Delete `crates/mempool/src/persistent.rs` (done in Task 07)
- Modify whirlpool-node imports
- Add generic type params to `PersistentTxPool`
- Change `TxSource` trait or its impl behavior
- Modify vendor/

## References

- **MIGRATION**: Step 4 — "Move PersistentTxPool"
- **IMPACT**: PersistentTxPool call site analysis
- **TESTS**: TB-008–TB-010
- **CHANGES**: `docs/refactor/mempool-split-interface/mempool-mdbx/CHANGES.md`

## Acceptance Criteria

- [ ] `PersistentTxPool` struct exists in `mempool-mdbx/src/persistent.rs`
- [ ] `PersistentTxPool` uses `MdbxMempoolStore` internally
- [ ] `impl TxSource for PersistentTxPool` works
- [ ] `PersistentTxPool` re-exported from `mempool-mdbx` lib.rs
- [ ] TB-008–TB-010 pass in mempool-mdbx
- [ ] All existing mempool tests still pass
- [ ] `nix develop --command cargo check --workspace` passes
- [ ] `nix develop --command cargo test --workspace` passes

## Post-Task Gate

```bash
nix develop --command cargo check --workspace
nix develop --command cargo test --workspace
```

Both must pass.

## Evidence

Record exit codes. Confirm TB-008–TB-010 equivalent tests appear in mempool-mdbx output.
