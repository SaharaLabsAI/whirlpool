# Task 03: Move Store Implementation to mempool-mdbx

| Field | Value |
|---|---|
| Status | `pending` |
| Dependencies | `02-add-mempool-store-trait` |
| Wave | 3 |
| Complexity | M (5+ files) |
| Target Crate(s) | `mempool-mdbx` (primary), `mempool` (error rename) |
| Migration Step | Step 3 |
| Change Type | MOVE + RENAME |

## Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

Tasks 01, 02 must be completed. `MempoolStore` trait must exist in `mempool`.

## Context

### Before
`MempoolStore` struct lives in `crates/mempool/src/store.rs` with 7 unit tests. `MempoolError::Mdbx(String)` variant exists.

### After
`MdbxMempoolStore` struct in `crates/mempool-mdbx/src/store.rs` implementing `mempool::MempoolStoreTrait`. Error variant renamed `Mdbx` → `Storage`. 7 unit tests + TN-001 in mempool-mdbx. Original struct in mempool still present (removed in Task 07).

## What to Do

### Phase 1: Tests

1. **Add TN-001** (trait implementation check) to the new store.rs test module:
   ```rust
   #[test]
   fn implements_mempool_store_trait() {
       fn _assert_impl<T: mempool::MempoolStoreTrait>() {}
       _assert_impl::<MdbxMempoolStore>();
   }
   ```

2. **Move TB-001 through TB-007** from `crates/mempool/src/store.rs` to `crates/mempool-mdbx/src/store.rs`. Update:
   - `MempoolStore::open(` → `MdbxMempoolStore::open(`
   - Any `use crate::` imports → use `mempool::MempoolError` and `crate::MdbxMempoolStore`

### Phase 2: Implement

1. **Update `crates/mempool-mdbx/Cargo.toml`** — Add dependencies:
   ```toml
   [dependencies]
   mempool = { path = "../mempool" }
   reth-libmdbx = { path = "../../vendor/reth/crates/storage/libmdbx-rs" }

   [dev-dependencies]
   tempfile = "3"
   ```

2. **Create `crates/mempool-mdbx/src/store.rs`** — Copy from `crates/mempool/src/store.rs`:
   - Rename struct: `MempoolStore` → `MdbxMempoolStore`
   - Update `use` statements: `use crate::error::MempoolError;` → `use mempool::MempoolError;`
   - Replace `From<reth_libmdbx::Error>` usage: where the code does `?` on reth_libmdbx::Error, use `.map_err(|e| MempoolError::Storage(e.to_string()))` instead.
   - Add trait impl:
     ```rust
     impl mempool::MempoolStoreTrait for MdbxMempoolStore {
         fn push(&self, tx: Vec<u8>) -> Result<(), MempoolError> {
             // delegate to existing push method body
         }
         fn drain_pending(&self) -> Result<Vec<Vec<u8>>, MempoolError> {
             // delegate to existing drain_pending method body
         }
     }
     ```
   - Keep `open()` as an inherent method (NOT in trait).
   - Include moved test module with updated references.

3. **Update `crates/mempool-mdbx/src/lib.rs`**:
   ```rust
   pub mod store;
   pub use store::MdbxMempoolStore;
   ```

4. **Rename error variant in `crates/mempool/src/error.rs`**:
   - `Mdbx(String)` → `Storage(String)`
   - Update `Display` impl: `"MDBX error: {}"` → `"Storage error: {}"`
   - Update `From<reth_libmdbx::Error>` impl to construct `Storage(...)` (this impl will be removed in Task 07)

5. **Update `crates/mempool/src/store.rs`** references — any `MempoolError::Mdbx(...)` → `MempoolError::Storage(...)`.

### Phase 3: Consumers
No consumer changes yet.

## Rollback

```bash
rm -rf crates/mempool-mdbx/src/store.rs
git checkout crates/mempool-mdbx/Cargo.toml crates/mempool-mdbx/src/lib.rs
git checkout crates/mempool/src/error.rs crates/mempool/src/store.rs
```

## Must NOT Do

- Delete `crates/mempool/src/store.rs` (done in Task 07)
- Modify `crates/mempool/src/persistent.rs` (done in Task 04)
- Modify whirlpool-node
- Add generic type params to any struct
- Modify vendor/

## References

- **MIGRATION**: Step 3 — "Move MempoolStore to mempool-mdbx"
- **IMPACT**: MempoolStore call site analysis, MempoolError variant rename
- **STRATEGY**: Error strategy (no orphan From impls)
- **TESTS**: TB-001–TB-007, TN-001
- **CHANGES**: `docs/refactor/mempool-split-interface/mempool-mdbx/CHANGES.md`

## Acceptance Criteria

- [ ] `MdbxMempoolStore` struct exists in `mempool-mdbx/src/store.rs`
- [ ] `MdbxMempoolStore` implements `mempool::MempoolStoreTrait`
- [ ] `MempoolError::Mdbx` renamed to `MempoolError::Storage` in mempool
- [ ] No `From<reth_libmdbx::Error>` impl in mempool-mdbx (uses `.map_err()` instead)
- [ ] TN-001 passes (trait impl check)
- [ ] TB-001–TB-007 pass in mempool-mdbx
- [ ] All existing mempool tests still pass (store.rs tests in mempool remain until Task 07)
- [ ] `nix develop --command cargo check --workspace` passes
- [ ] `nix develop --command cargo test --workspace` passes

## Post-Task Gate

```bash
nix develop --command cargo check --workspace
nix develop --command cargo test --workspace
```

Both must pass.

## Evidence

Record exit codes. Confirm TN-001 and TB-001–TB-007 equivalent tests appear in mempool-mdbx test output.
