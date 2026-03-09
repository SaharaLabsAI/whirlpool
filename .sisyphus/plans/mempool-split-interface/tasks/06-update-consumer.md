# Task 06: Update whirlpool-node Consumer

| Field | Value |
|---|---|
| Status | `completed` |
| Dependencies | `05-move-integration-tests` |
| Wave | 6 |
| Complexity | S (2 files) |
| Target Crate(s) | `whirlpool-node` |
| Migration Step | Step 6 |
| Change Type | MOVE (import path) |

## Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

Tasks 01–05 must be completed. `mempool-mdbx` must be fully functional with all types and tests.

## Context

### Before
`whirlpool-node` depends on `mempool` and imports `mempool::PersistentTxPool`.

### After
`whirlpool-node` depends on `mempool-mdbx` and imports `mempool_mdbx::PersistentTxPool`. No behavior change.

## What to Do

### Phase 1: Tests
No new tests. Existing whirlpool-node functionality verified by workspace build.

### Phase 2: Implement

1. **Update `crates/whirlpool-node/Cargo.toml`**:
   - Add: `mempool-mdbx = { path = "../mempool-mdbx" }`
   - Keep `mempool` dep for now (may be needed by other imports; removed in Task 07 if unused)
   
   Actually, check: if whirlpool-node only uses `PersistentTxPool` from mempool, it can switch entirely:
   - Replace: `mempool = { path = "../mempool" }` → `mempool-mdbx = { path = "../mempool-mdbx" }`

2. **Update `crates/whirlpool-node/src/main.rs`**:
   - Change: `use mempool::PersistentTxPool;` → `use mempool_mdbx::PersistentTxPool;`
   - No other changes needed — `PersistentTxPool::open()` and `Arc<dyn TxSource>` usage remain identical.

### Phase 3: Consumers
This task IS the consumer update.

## Rollback

```bash
git checkout crates/whirlpool-node/Cargo.toml crates/whirlpool-node/src/main.rs
```

## Must NOT Do

- Modify PersistentTxPool behavior
- Add mempool-mdbx as dep AND keep mempool dep (unless mempool is used for other imports)
- Change any logic in main.rs beyond the import path
- Modify any other crate
- Modify vendor/

## References

- **MIGRATION**: Step 6 — "Update whirlpool-node"
- **IMPACT**: PersistentTxPool external consumer analysis
- **CHANGES**: `docs/refactor/mempool-split-interface/mempool/CHANGES.md` (consumer migration)

## Acceptance Criteria

- [ ] `whirlpool-node/Cargo.toml` depends on `mempool-mdbx` (not `mempool`)
- [ ] `whirlpool-node/src/main.rs` imports from `mempool_mdbx`
- [ ] `nix develop --command cargo check --workspace` passes
- [ ] `nix develop --command cargo test --workspace` passes

## Post-Task Gate

```bash
nix develop --command cargo check --workspace
nix develop --command cargo test --workspace
```

Both must pass.

## Evidence

Record exit codes. Confirm whirlpool-node builds with mempool-mdbx dep.
