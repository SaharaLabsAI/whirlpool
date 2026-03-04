## `09-compatibility-export-cleanup`

> Remove transitional re-exports and enforce canonical-only trait paths.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `08-node-consumer-import-migration` |
| **Wave** | 3 |
| **Complexity** | M |
| **Target Crate(s)** | `consensus`, `app`, `consensus-simplex`, `app-evm` (source); workspace consumers |
| **Migration Step** | #9 |
| **Change Type** | delete |

### Pre-Task Gate

```bash
cargo check --workspace
cargo test --workspace
```

### Context

Cleanup is intentionally last: all consumers should already use canonical paths. This step removes temporary compatibility re-exports and stale legacy path references.

### What to do

#### Phase 1 - Update/create test expectations

1. Add/adjust `TB-012` and `TN-010` post-cleanup legacy path failure checks.

```bash
cargo check --workspace
```

#### Phase 2 - Implement the change

2. Remove compatibility re-exports from `consensus`, `app`, `consensus-simplex`, and `app-evm` crate roots/modules.
3. Update docs/comments/tests that still reference legacy trait paths.

```bash
cargo check --workspace
cargo test --workspace
```

### Rollback

```bash
git restore crates/consensus/src crates/app/src crates/consensus-simplex/src crates/app-evm/src docs
```

**Rollback dependencies**: if cleanup fails, revert this task first; upstream tasks can remain applied.

### Mock Boundary

**Allowed to mock**: none.
**Must NOT mock**: workspace compilation and test gates.

### References

- `docs/refactor/split-interface-implementation/MIGRATION.md` Step 9
- `docs/refactor/split-interface-implementation/TESTS.md` (`TB-012`, `TN-010`)

### Acceptance Criteria

```bash
cargo check --workspace
cargo test --workspace
```

Evidence: `.sisyphus/plans/split-interface-from-implementation/evidence/09-compatibility-export-cleanup.log`
