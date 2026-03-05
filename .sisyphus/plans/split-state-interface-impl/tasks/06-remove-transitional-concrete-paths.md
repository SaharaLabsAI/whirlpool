## `06-remove-transitional-concrete-paths`

> Remove `state` concrete re-exports and finalize interface/implementation separation after all consumer rewiring is complete.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | 05-rewire-whirlpool-node-wrapper |
| **Wave** | 6 |
| **Complexity** | M |
| **Target Crate(s)** | `state` (source), `state-memory` (target), `app-evm`/`whirlpool-node` (consumers) |
| **Migration Step** | #6 |
| **Change Type** | delete |

### Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

### Context

This cleanup step enforces the final boundary by removing transitional concrete paths from `state`. It should happen only after both consumers compile on `state-memory` concrete imports.

**Before state**: `state` still exposes concrete compatibility paths.
**After state**: `state` is interface-only (`StateDb`, `StateError`); concrete paths live only in `state-memory`.

### What to do

#### Phase 1 - Update/create test expectations

1. Add/adjust guards for `TB-006` and `TN-006` to detect stale `state::{InMemoryStateDb, DbAccount}` paths.

```bash
nix develop --command cargo check -p state -p state-memory -p app-evm -p whirlpool-node
```

#### Phase 2 - Implement the change

2. Remove concrete re-exports from `crates/state/src/lib.rs`.
3. Update remaining stale path references in affected consumer/test/doc files.
4. Keep interface/shared exports in `state` unchanged.

```bash
nix develop --command cargo check -p state -p state-memory -p app-evm -p whirlpool-node
nix develop --command cargo test -p state-memory --lib
rg "state::(InMemoryStateDb|DbAccount)" crates/app-evm crates/whirlpool-node crates/state
```

### Rollback

```bash
git restore crates/state/src/lib.rs crates/state/src/db.rs crates/app-evm crates/whirlpool-node
nix develop --command cargo check -p state -p app-evm -p whirlpool-node
```

**Rollback dependencies**: if Step 6 rollback requires compatibility re-exports, ensure Task 05 state is preserved first.

### References

- `docs/refactor/split-state-interface-impl/MIGRATION.md` Step 6
- `docs/refactor/split-state-interface-impl/TESTS.md` (`TB-006`, `TN-006`)
- `docs/refactor/split-state-interface-impl/state/CHANGES.md`

### Acceptance Criteria

```bash
nix develop --command cargo check -p state -p state-memory -p app-evm -p whirlpool-node
nix develop --command cargo test -p state-memory --lib
```

Evidence: `.sisyphus/plans/split-state-interface-impl/evidence/06-remove-transitional-concrete-paths.log`
