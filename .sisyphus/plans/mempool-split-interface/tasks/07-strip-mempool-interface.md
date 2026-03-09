# Task 07: Strip mempool to Interface-Only

| Field | Value |
|---|---|
| Status | `completed` |
| Dependencies | `06-update-consumer` |
| Wave | 7 |
| Complexity | M (5+ files) |
| Target Crate(s) | `mempool` |
| Migration Step | Step 7 |
| Change Type | DELETE + RESTRUCTURE |

## Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

Tasks 01–06 must be completed. No consumer uses `mempool::PersistentTxPool` or `mempool::MempoolStore` (struct) anymore.

## Context

### Before
`mempool` still has `store.rs`, `persistent.rs`, tests, and deps on `app` + `reth-libmdbx` alongside the new `traits.rs`. Duplicate code exists in both mempool and mempool-mdbx.

### After
`mempool` is interface-only: `traits.rs` (MempoolStore trait) + `error.rs` (MempoolError). No `store.rs`, no `persistent.rs`, no integration tests, no `reth-libmdbx` dep.

## What to Do

### Phase 1: Tests

Verify TN-002 (trait object safety test in traits.rs) still passes after cleanup.

### Phase 2: Implement

1. **Delete `crates/mempool/src/store.rs`**

2. **Delete `crates/mempool/src/persistent.rs`**

3. **Delete `crates/mempool/tests/` directory** (if not already removed in Task 05)

4. **Update `crates/mempool/src/lib.rs`** — Replace entire content:
   ```rust
   pub mod error;
   pub mod traits;

   pub use error::MempoolError;
   pub use traits::MempoolStore;
   ```
   
   **Note**: `MempoolStoreTrait` alias is gone. The trait is now `MempoolStore` directly (no name collision since the struct is gone).

5. **Update `crates/mempool/src/error.rs`**:
   - Remove `From<reth_libmdbx::Error>` impl (no longer needed in interface crate; mempool-mdbx uses `.map_err()`)
   - Remove `use reth_libmdbx;` import if present
   - Keep `From<std::io::Error>` impl (generic, stays in interface)
   - Keep `MempoolError::Storage(String)` variant
   - Keep `MempoolError::Io(std::io::Error)` variant

6. **Update `crates/mempool/Cargo.toml`**:
   - Remove `app` dependency (trait doesn't reference app::TxSource)
   - Remove `reth-libmdbx` dependency
   - Remove `tempfile` dev-dependency
   - Final deps should be minimal (possibly just `thiserror` or std Error impls)

7. **Update `crates/mempool-mdbx/Cargo.toml`** if needed — ensure it depends on `mempool` (for trait + error). It should already from Task 03.

8. **Update `crates/mempool-mdbx/src/store.rs`** — Change any `use mempool::MempoolStoreTrait` → `use mempool::MempoolStore` (since the alias is removed and the trait is now the primary export name).

9. **Update `crates/mempool-mdbx/src/persistent.rs`** — Same: update trait import name if it referenced `MempoolStoreTrait`.

### Phase 3: Consumers
Verify no remaining references to deleted modules across workspace.

## Rollback

```bash
git checkout crates/mempool/
git checkout crates/mempool-mdbx/src/store.rs crates/mempool-mdbx/src/persistent.rs crates/mempool-mdbx/Cargo.toml
```

## Must NOT Do

- Delete `error.rs` (stays in interface)
- Delete `traits.rs` (stays in interface)
- Modify mempool-mdbx test logic
- Modify whirlpool-node
- Modify vendor/

## References

- **MIGRATION**: Step 7 — "Strip mempool to interface-only"
- **IMPACT**: Full symbol analysis
- **STRATEGY**: Final interface crate structure
- **TESTS**: TN-002 (must still pass)
- **CHANGES**: `docs/refactor/mempool-split-interface/mempool/CHANGES.md`

## Acceptance Criteria

- [ ] `crates/mempool/src/store.rs` does NOT exist
- [ ] `crates/mempool/src/persistent.rs` does NOT exist
- [ ] `crates/mempool/tests/` does NOT exist
- [ ] `crates/mempool/Cargo.toml` has NO `reth-libmdbx` or `app` dependency
- [ ] `crates/mempool/src/lib.rs` exports only `MempoolError` and `MempoolStore` (trait)
- [ ] `mempool::MempoolStore` is the trait (not the old struct)
- [ ] TN-002 passes
- [ ] All mempool-mdbx tests still pass (16 moved + TN-001)
- [ ] `nix develop --command cargo check --workspace` passes
- [ ] `nix develop --command cargo test --workspace` passes

## Post-Task Gate

```bash
nix develop --command cargo check --workspace
nix develop --command cargo test --workspace
```

Both must pass. Confirm mempool has zero storage dependencies.

## Post-Task Reconciliation

After this task:
- Verify `cargo tree -p mempool` shows minimal deps (no reth-libmdbx)
- Verify `cargo tree -p mempool-mdbx` shows mempool + reth-libmdbx
- Pattern matches state/state-memory split

## Evidence

Record exit codes. Record `cargo tree -p mempool` output showing clean interface deps.
