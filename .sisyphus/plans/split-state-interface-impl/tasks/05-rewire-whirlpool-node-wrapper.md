## `05-rewire-whirlpool-node-wrapper`

> Rewire `whirlpool-node` `TestStateDb` wrapper to `state_memory::InMemoryStateDb` while preserving `state::StateError` contracts.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | 04-rewire-app-evm-to-state-memory |
| **Wave** | 5 |
| **Complexity** | S |
| **Target Crate(s)** | `whirlpool-node` (consumer), `state-memory` (target) |
| **Migration Step** | #5 |
| **Change Type** | move |

### Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

### Context

This step handles node runtime wrapper rewiring after `app-evm` has migrated. The concrete DB path changes, but error and trait contracts remain in `state`.

**Before state**: `TestStateDb` wraps `state::InMemoryStateDb`.
**After state**: `TestStateDb` wraps `state_memory::InMemoryStateDb`.

### What to do

#### Phase 1 - Update/create test expectations

1. Add/adjust checks mapped to `TB-005` and `TN-005` for wrapper delegation behavior.

```bash
nix develop --command cargo check -p whirlpool-node
```

#### Phase 2 - Implement the change

2. Update `crates/whirlpool-node/Cargo.toml` to include `state-memory` dependency.
3. Update concrete import path in `crates/whirlpool-node/src/main.rs` to `state_memory::InMemoryStateDb`.
4. Keep `type Error = state::StateError` and delegation semantics unchanged.

```bash
nix develop --command cargo check -p whirlpool-node
```

### Rollback

```bash
git restore crates/whirlpool-node/Cargo.toml crates/whirlpool-node/src/main.rs
nix develop --command cargo check -p whirlpool-node
```

**Rollback dependencies**: revert Task 06 first if already applied.

### References

- `docs/refactor/split-state-interface-impl/MIGRATION.md` Step 5
- `docs/refactor/split-state-interface-impl/TESTS.md` (`TB-005`, `TN-005`)
- `docs/refactor/split-state-interface-impl/whirlpool-node/CHANGES.md`

### Acceptance Criteria

```bash
nix develop --command cargo check -p whirlpool-node
```

Evidence: `.sisyphus/plans/split-state-interface-impl/evidence/05-rewire-whirlpool-node-wrapper.log`
