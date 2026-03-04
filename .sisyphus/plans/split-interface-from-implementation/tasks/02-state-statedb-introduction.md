## `02-state-statedb-introduction`

> Introduce `state::traits::StateDb` and implement it for `InMemoryStateDb`.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `01-consensus-traits-boundary` |
| **Wave** | 1 |
| **Complexity** | M |
| **Target Crate(s)** | `state` (target), `app-evm` (consumer) |
| **Migration Step** | #2 |
| **Change Type** | introduce |

### Pre-Task Gate

```bash
cargo check --workspace
```

### Context

This foundation step introduces the missing state interface boundary while preserving concrete exports. Downstream `app-evm` should shift to trait bounds without behavior changes.

### What to do

#### Phase 1 - Update/create test expectations

1. Add/adjust `TB-003`, `TB-004` checks for trait-bound compilation.
2. Add `TN-002`, `TN-003` contract checks for `state_root` and `commit` semantics.

```bash
cargo test -p state
```

#### Phase 2 - Implement the change

3. Add `crates/state/src/traits.rs` with `StateDb` trait.
4. Implement `StateDb` for `InMemoryStateDb` in `crates/state/src/db.rs`.
5. Expose `pub mod traits; pub use traits::StateDb;` in `crates/state/src/lib.rs`.

```bash
cargo check -p state && cargo check -p app-evm
```

#### Phase 3 - Update consumers (if applicable)

6. Update `app-evm` imports to use `state::traits::StateDb` where needed.

```bash
cargo check --workspace
```

### Rollback

```bash
git restore crates/state/src/lib.rs crates/state/src/db.rs crates/app-evm/src/executor.rs
rm -f crates/state/src/traits.rs
```

**Rollback dependencies**: revert this task before reverting task `01`.

### Mock Boundary

**Allowed to mock**: test-only state fixtures.
**Must NOT mock**: `InMemoryStateDb` commit/state-root behavior.

### References

- `docs/refactor/split-interface-implementation/MIGRATION.md` Step 2
- `docs/refactor/split-interface-implementation/TESTS.md` (`TB-003`, `TB-004`, `TN-002`, `TN-003`)
- `docs/refactor/split-interface-implementation/state/CHANGES.md`

### Acceptance Criteria

```bash
cargo check -p state -p app-evm
cargo test -p state
```

Evidence: `.sisyphus/plans/split-interface-from-implementation/evidence/02-state-statedb-introduction.log`
