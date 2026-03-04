## `08-node-consumer-import-migration`

> Migrate node crates to canonical trait paths after adapter boundaries are in place.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `07-p2p-commonware-transport-introduction` |
| **Wave** | 3 |
| **Complexity** | M |
| **Target Crate(s)** | `whirlpool-node` (consumer), `whirlpool-node-simple` (consumer) |
| **Migration Step** | #8 |
| **Change Type** | restructure |

### Pre-Task Gate

```bash
cargo check --workspace
```

### Context

This step moves final consumers to canonical paths while compatibility shims still exist. It should avoid introducing new old-path imports.

### What to do

#### Phase 1 - Update/create test expectations

1. Add/adjust `TB-010`, `TB-011`, and `TN-009` import guards.

```bash
cargo test -p whirlpool-node -p whirlpool-node-simple
```

#### Phase 2 - Implement the change

2. Update imports in `crates/whirlpool-node/**` to canonical trait paths.
3. Update imports in `crates/whirlpool-node-simple/**` similarly.
4. Remove any newly introduced legacy-path imports.

```bash
cargo check -p whirlpool-node && cargo check -p whirlpool-node-simple
```

### Rollback

```bash
git restore crates/whirlpool-node crates/whirlpool-node-simple
```

**Rollback dependencies**: revert this task before reverting task `07`.

### Mock Boundary

**Allowed to mock**: node test harness fixtures.
**Must NOT mock**: compile-level import resolution across crates.

### References

- `docs/refactor/split-interface-implementation/MIGRATION.md` Step 8
- `docs/refactor/split-interface-implementation/TESTS.md` (`TB-010`, `TB-011`, `TN-009`)

### Acceptance Criteria

```bash
cargo check -p whirlpool-node -p whirlpool-node-simple
cargo test -p whirlpool-node -p whirlpool-node-simple
```

Evidence: `.sisyphus/plans/split-interface-from-implementation/evidence/08-node-consumer-import-migration.log`
