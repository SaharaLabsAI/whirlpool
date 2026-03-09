# Task 05: Move Integration Tests to mempool-mdbx

| Field | Value |
|---|---|
| Status | `pending` |
| Dependencies | `04-move-persistent-txpool` |
| Wave | 5 |
| Complexity | S (2 files) |
| Target Crate(s) | `mempool-mdbx`, `mempool` |
| Migration Step | Step 5 |
| Change Type | MOVE |

## Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

Tasks 01–04 must be completed. Both `MdbxMempoolStore` and `PersistentTxPool` must exist in `mempool-mdbx`.

## Context

### Before
6 integration tests in `crates/mempool/tests/integration.rs` import `mempool::PersistentTxPool` and `app::traits::TxSource`.

### After
Same 6 tests in `crates/mempool-mdbx/tests/integration.rs` importing `mempool_mdbx::PersistentTxPool`. Old file deleted.

## What to Do

### Phase 1: Tests

This task IS the test migration. TB-011 through TB-016.

### Phase 2: Implement

1. **Create `crates/mempool-mdbx/tests/integration.rs`** — Copy from `crates/mempool/tests/integration.rs`:
   - Change `use mempool::PersistentTxPool;` → `use mempool_mdbx::PersistentTxPool;`
   - `use app::traits::TxSource;` stays the same
   - `use std::sync::Arc;` stays the same
   - All test function bodies remain identical

2. **Update `crates/mempool-mdbx/Cargo.toml`** — Ensure dev-dependencies include:
   ```toml
   [dev-dependencies]
   tempfile = "3"
   app = { path = "../app" }
   ```
   (Note: `app` may already be in `[dependencies]` from Task 04, in which case it's available for tests too.)

3. **Delete `crates/mempool/tests/integration.rs`** — The original integration tests now live in mempool-mdbx.

4. **Delete `crates/mempool/tests/` directory** if empty after removing integration.rs.

### Phase 3: Consumers
No consumer changes.

## Rollback

```bash
rm -rf crates/mempool-mdbx/tests/
git checkout crates/mempool/tests/integration.rs
```

## Must NOT Do

- Modify test logic or assertions
- Add new tests (beyond moving existing)
- Modify whirlpool-node
- Modify any source files (only test files)
- Modify vendor/

## References

- **MIGRATION**: Step 5 — "Move integration tests"
- **TESTS**: TB-011–TB-016 (all import_path_changed)
- **CHANGES**: `docs/refactor/mempool-split-interface/mempool-mdbx/CHANGES.md`

## Acceptance Criteria

- [ ] `crates/mempool-mdbx/tests/integration.rs` exists with 6 tests
- [ ] `crates/mempool/tests/integration.rs` does NOT exist
- [ ] All 6 integration tests pass in mempool-mdbx
- [ ] `nix develop --command cargo check --workspace` passes
- [ ] `nix develop --command cargo test --workspace` passes

## Post-Task Gate

```bash
nix develop --command cargo check --workspace
nix develop --command cargo test --workspace
```

Both must pass. All 6 integration tests must appear in mempool-mdbx test output.

## Evidence

Record exit codes. Grep test output for the 6 test names in mempool_mdbx.
