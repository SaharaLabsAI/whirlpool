## `03-p2p-traits-stabilization`

> Keep `p2p::traits` interface-only and stable for adapter migration.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `02-state-statedb-introduction` |
| **Wave** | 1 |
| **Complexity** | S |
| **Target Crate(s)** | `p2p` (both), `p2p-commonware` (consumer) |
| **Migration Step** | #3 |
| **Change Type** | restructure |

### Pre-Task Gate

```bash
cargo check --workspace
```

### Context

`p2p` is already close to desired structure. This step is a stabilization gate to prevent concrete leakage into interface modules and preserve exports for downstream crates.

### What to do

#### Phase 1 - Update/create test expectations

1. Add/refresh checks for `TB-005` and `TN-004` in `crates/p2p/src/traits.rs` tests.

```bash
cargo test -p p2p
```

#### Phase 2 - Implement the change

2. Move any concrete/helper items out of `crates/p2p/src/traits.rs` if present.
3. Keep stable re-exports in `crates/p2p/src/lib.rs`.

```bash
cargo check -p p2p && cargo check -p p2p-commonware
```

### Rollback

```bash
git restore crates/p2p/src/traits.rs crates/p2p/src/lib.rs
```

**Rollback dependencies**: revert this task before reverting task `02`.

### Mock Boundary

**Allowed to mock**: network test doubles already used by p2p tests.
**Must NOT mock**: trait object/interface surface expectations.

### References

- `docs/refactor/split-interface-implementation/MIGRATION.md` Step 3
- `docs/refactor/split-interface-implementation/TESTS.md` (`TB-005`, `TN-004`)
- `docs/refactor/split-interface-implementation/p2p/CHANGES.md`

### Acceptance Criteria

```bash
cargo check -p p2p -p p2p-commonware
cargo test -p p2p
```

Evidence: `.sisyphus/plans/split-interface-from-implementation/evidence/03-p2p-traits-stabilization.log`
