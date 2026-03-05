## `04-rewire-app-evm-to-state-memory`

> Rewire `app-evm` concrete DB imports/dependencies to `state-memory` while keeping interface trait bounds on `state`.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | 03-move-concrete-db-and-revm-impls |
| **Wave** | 4 |
| **Complexity** | L |
| **Target Crate(s)** | `app-evm` (consumer), `state-memory` (target) |
| **Migration Step** | #4 |
| **Change Type** | move |

### Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

### Context

`app-evm` is the largest concrete consumer surface and has both runtime and test imports. This step migrates concrete paths only; trait contracts continue to come from `state::traits::StateDb`.

**Before state**: `app-evm` imports `state::InMemoryStateDb`.
**After state**: `app-evm` imports `state_memory::InMemoryStateDb`.

### What to do

#### Phase 1 - Update/create test expectations

1. Update test modules tied to `TB-004` and `TN-004` to reference `state_memory::InMemoryStateDb`.
2. Keep trait-bound assertions on `state::traits::StateDb` untouched.

```bash
nix develop --command cargo test -p app-evm
```

#### Phase 2 - Implement the change

3. Update `crates/app-evm/Cargo.toml` to include `state-memory` dependency.
4. Replace concrete imports in `crates/app-evm/src/executor.rs` and tests.
5. Preserve runtime behavior and execution semantics.

```bash
nix develop --command cargo check -p app-evm
nix develop --command cargo test -p app-evm
```

### Rollback

```bash
git restore crates/app-evm/Cargo.toml crates/app-evm/src/executor.rs crates/app-evm/tests/application_integration.rs crates/app-evm/tests/cross_crate_flows.rs crates/app-evm/tests/evm_execution_integration.rs crates/app-evm/tests/integration.rs
nix develop --command cargo check -p app-evm
```

**Rollback dependencies**: revert Task 05 and Task 06 first if they depend on this rewire.

### References

- `docs/refactor/split-state-interface-impl/MIGRATION.md` Step 4
- `docs/refactor/split-state-interface-impl/TESTS.md` (`TB-004`, `TN-004`)
- `docs/refactor/split-state-interface-impl/app-evm/CHANGES.md`

### Acceptance Criteria

```bash
nix develop --command cargo check -p app-evm
nix develop --command cargo test -p app-evm
```

Evidence: `.sisyphus/plans/split-state-interface-impl/evidence/04-rewire-app-evm-to-state-memory.log`
