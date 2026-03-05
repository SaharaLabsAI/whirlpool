## `03-move-concrete-db-and-revm-impls`

> Move `DbAccount`, `InMemoryStateDb`, and revm database impl blocks from `state` into `state-memory` with behavior parity.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | 02-scaffold-state-memory-crate |
| **Wave** | 3 |
| **Complexity** | L |
| **Target Crate(s)** | `state` (source), `state-memory` (target) |
| **Migration Step** | #3 |
| **Change Type** | move |

### Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

### Context

This is the highest-risk movement step: concrete behavior and `revm` integration change physical crate ownership but must stay semantically identical. Keep constructor, storage, commit, and state-root behavior unchanged.

**Before state**: concrete DB implementation is in `crates/state/src/db.rs`.
**After state**: concrete DB implementation lives in `crates/state-memory/src/db.rs`.

### What to do

#### Phase 1 - Update/create test expectations

1. Port/add parity checks for `TB-003` and `TN-003` under `state-memory` tests.
2. Ensure compile expectations fail until impl blocks are moved with the concrete type.

```bash
nix develop --command cargo test -p state-memory --lib
```

#### Phase 2 - Implement the change

3. Move `DbAccount` and `InMemoryStateDb` definitions to `state_memory::db`.
4. Move `impl DatabaseRef` and `impl Database` blocks with `StateError` linkage preserved.
5. Remove/adjust duplicated concrete definitions in `state` and keep interface-only contracts there.

```bash
nix develop --command cargo check -p state-memory
nix develop --command cargo check -p state -p state-memory
nix develop --command cargo test -p state-memory --lib
```

### Rollback

```bash
git restore crates/state/src/db.rs crates/state/src/lib.rs crates/state-memory/src/db.rs crates/state-memory/src/lib.rs
nix develop --command cargo check -p state -p state-memory
```

**Rollback dependencies**: reverting Task 03 requires Task 02 scaffolding to exist; revert Task 04+ first if already applied.

### References

- `docs/refactor/split-state-interface-impl/MIGRATION.md` Step 3
- `docs/refactor/split-state-interface-impl/TESTS.md` (`TB-003`, `TN-003`)
- `docs/refactor/split-state-interface-impl/state/CHANGES.md`
- `docs/refactor/split-state-interface-impl/state-memory/CHANGES.md`

### Acceptance Criteria

```bash
nix develop --command cargo check -p state-memory
nix develop --command cargo test -p state-memory --lib
```

Evidence: `.sisyphus/plans/split-state-interface-impl/evidence/03-move-concrete-db-and-revm-impls.log`
