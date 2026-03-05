## `01-lock-interface-surface-in-state`

> Keep `state` as canonical interface/error contract crate before any concrete-symbol movement.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | none |
| **Wave** | 1 |
| **Complexity** | S |
| **Target Crate(s)** | `state` (source/target) |
| **Migration Step** | #1 |
| **Change Type** | restructure |

### Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

### Context

This step locks the contract boundary first so downstream rewiring can happen without trait/error churn. It preserves the interface symbols in `state` and prevents adding fresh concrete exports during migration.

**Before state**: `state` root exports both interface and concrete symbols.
**After state**: interface contracts remain explicit and stable in `state`.

### What to do

#### Phase 1 - Update/create test expectations

1. Add/adjust compile coverage for `TB-001`, `TN-001`, and `TN-002` in `crates/state/src/{lib.rs,error.rs,traits.rs}` test modules.
2. Assert `StateDb` and `StateError` stay reachable from canonical `state::*` paths.

```bash
nix develop --command cargo test -p state --lib
```

#### Phase 2 - Implement the change

3. Ensure `crates/state/src/lib.rs` exports only the intended interface surface for this phase.
4. Keep `DBErrorMarker` bound to `StateError` in `crates/state/src/error.rs`.
5. Avoid adding new concrete exports from `state`.

```bash
nix develop --command cargo check -p state
```

### Rollback

```bash
git restore crates/state/src/lib.rs crates/state/src/traits.rs crates/state/src/error.rs
nix develop --command cargo check -p state
```

**Rollback dependencies**: none.

### References

- `docs/refactor/split-state-interface-impl/MIGRATION.md` Step 1
- `docs/refactor/split-state-interface-impl/TESTS.md` (`TB-001`, `TN-001`, `TN-002`)
- `docs/refactor/split-state-interface-impl/state/CHANGES.md`

### Acceptance Criteria

```bash
nix develop --command cargo check -p state
nix develop --command cargo test -p state --lib
```

Evidence: `.sisyphus/plans/split-state-interface-impl/evidence/01-lock-interface-surface-in-state.log`
