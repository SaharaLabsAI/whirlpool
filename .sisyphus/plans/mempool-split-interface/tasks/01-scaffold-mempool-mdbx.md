# Task 01: Scaffold mempool-mdbx Crate

| Field | Value |
|---|---|
| Status | `completed` |
| Dependencies | None |
| Wave | 1 |
| Complexity | S (3 files) |
| Target Crate(s) | `mempool-mdbx` (new), workspace root |
| Migration Step | Step 1 |
| Change Type | CREATE |

## Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

Must pass before starting.

## Context

### Before
No `mempool-mdbx` crate exists. All mempool code lives in `crates/mempool/`.

### After
Empty `crates/mempool-mdbx/` crate registered in workspace. Builds with no code.

## What to Do

### Phase 1: Tests
No tests for this task — pure scaffolding.

### Phase 2: Implement

1. **Create `crates/mempool-mdbx/Cargo.toml`**:
   ```toml
   [package]
   name = "mempool-mdbx"
   edition = "2021"

   [dependencies]
   ```
   Start minimal — deps added in later tasks.

2. **Create `crates/mempool-mdbx/src/lib.rs`**:
   ```rust
   // MDBX-backed mempool implementation.
   ```

3. **Add to workspace** — In root `Cargo.toml`, add `"crates/mempool-mdbx"` to `[workspace].members` list.

### Phase 3: Consumers
No consumer changes.

## Rollback

```bash
rm -rf crates/mempool-mdbx
# Remove "crates/mempool-mdbx" from root Cargo.toml [workspace].members
git checkout Cargo.toml
```

## Must NOT Do

- Add any dependencies yet (done in Task 03)
- Add any source code beyond empty lib.rs
- Modify any existing crate
- Modify vendor/

## References

- **MIGRATION**: Step 1 — "Create crate scaffold"
- **STRATEGY**: Scaffolding approach
- **CHANGES**: `docs/refactor/mempool-split-interface/mempool-mdbx/CHANGES.md`

## Acceptance Criteria

- [ ] `crates/mempool-mdbx/Cargo.toml` exists with correct package name
- [ ] `crates/mempool-mdbx/src/lib.rs` exists
- [ ] `"crates/mempool-mdbx"` appears in workspace members
- [ ] `nix develop --command cargo check --workspace` passes
- [ ] `nix develop --command cargo test --workspace` passes

## Post-Task Gate

```bash
nix develop --command cargo check --workspace
nix develop --command cargo test --workspace
```

Both must pass.

## Evidence

Record exit codes of both cargo commands.
