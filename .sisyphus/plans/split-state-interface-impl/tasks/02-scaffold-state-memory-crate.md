## `02-scaffold-state-memory-crate`

> Create `state-memory` crate scaffolding, workspace membership, and root concrete re-exports.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | 01-lock-interface-surface-in-state |
| **Wave** | 2 |
| **Complexity** | M |
| **Target Crate(s)** | `state-memory` (target), `workspace` (consumer) |
| **Migration Step** | #2 |
| **Change Type** | move |

### Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

### Context

This step introduces the new concrete implementation crate while preserving layering (`state-memory -> state`). It must compile independently before moving implementation logic or touching consumers.

**Before state**: no `state-memory` crate exists.
**After state**: new crate is wired and exports `DbAccount`/`InMemoryStateDb` entrypoints.

### What to do

#### Phase 1 - Update/create test expectations

1. Add/adjust compile expectation for `TB-002` that fails until crate/workspace wiring is complete.

```bash
nix develop --command cargo check -p state-memory
```

#### Phase 2 - Implement the change

2. Add `crates/state-memory/Cargo.toml` with `state` dependency and required `revm`/`alloy-genesis` dependencies.
3. Add `crates/state-memory/src/lib.rs` with public module/export surface.
4. Add `crates/state-memory` to workspace members in root `Cargo.toml`.

```bash
nix develop --command cargo check -p state-memory
nix develop --command cargo metadata --no-deps
```

### Rollback

```bash
git restore Cargo.toml
rm -rf crates/state-memory
nix develop --command cargo check -p state
```

**Rollback dependencies**: revert Task 02 before attempting to revert Task 03+.

### References

- `docs/refactor/split-state-interface-impl/MIGRATION.md` Step 2
- `docs/refactor/split-state-interface-impl/TESTS.md` (`TB-002`)
- `docs/refactor/split-state-interface-impl/state-memory/CHANGES.md`

### Acceptance Criteria

```bash
nix develop --command cargo check -p state-memory
nix develop --command cargo metadata --no-deps
```

Evidence: `.sisyphus/plans/split-state-interface-impl/evidence/02-scaffold-state-memory-crate.log`
