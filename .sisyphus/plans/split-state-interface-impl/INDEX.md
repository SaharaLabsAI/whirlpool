# split-state-interface-impl - Execution Plan

## TL;DR

| Field | Value |
|-------|-------|
| **Summary** | Split `state` into interface contracts (`state`) and concrete in-memory implementation (`state-memory`) while preserving behavior |
| **Deliverables** | New `state-memory` crate, migrated concrete imports in `app-evm` and `whirlpool-node`, interface-only `state` exports |
| **Effort** | 6 tasks, 6 waves, estimated L complexity |
| **Critical path** | `01` -> `02` -> `03` -> `04` -> `05` -> `06` |
| **Rollback** | Each task has scoped rollback; full rollback proceeds reverse order |

## Context

**Design source**: `docs/refactor/split-state-interface-impl/`
**Migration step source**: `docs/refactor/split-state-interface-impl/MIGRATION.md` (6 steps)

### Refactor Contract Table (excerpt)

| Crate | Role | Change Type | Risk Level |
|-------|------|------------|------------|
| `state` | source/target | restructure + export cleanup | medium |
| `state-memory` | target | create + move concrete implementation | high |
| `app-evm` | consumer | import/dependency rewire | high |
| `whirlpool-node` | consumer | import/dependency rewire | medium |

### Current Implementation Anchors

- Concrete DB implementation: `crates/state/src/db.rs`
- Interface contracts: `crates/state/src/traits.rs`, `crates/state/src/error.rs`
- App consumer seam: `crates/app-evm/src/executor.rs`
- Node wrapper seam: `crates/whirlpool-node/src/main.rs`

## Work Objectives

### Definition of Done

```bash
nix develop --command cargo check -p state -p state-memory -p app-evm -p whirlpool-node
nix develop --command cargo test -p state-memory --lib
```

### Must NOT Have

- No modifications under `vendor/` or `third_party/`
- No reverse dependency edge `state -> state-memory`
- No behavior changes to `InMemoryStateDb` semantics (move-only refactor)
- No lingering concrete import paths `state::{InMemoryStateDb,DbAccount}` after Task 06

## Verification Strategy

- Per-task evidence logs: `.sisyphus/plans/split-state-interface-impl/evidence/NN-<slug>.log`
- Migration-step verification commands are copied from design docs and use `nix develop --command` wrappers
- Rollback sections are mandatory and include dependency notes

## Execution Strategy

### Dependency Matrix

| Task | Depends On | Wave |
|------|------------|------|
| 01-lock-interface-surface-in-state | none | 1 |
| 02-scaffold-state-memory-crate | 01-lock-interface-surface-in-state | 2 |
| 03-move-concrete-db-and-revm-impls | 02-scaffold-state-memory-crate | 3 |
| 04-rewire-app-evm-to-state-memory | 03-move-concrete-db-and-revm-impls | 4 |
| 05-rewire-whirlpool-node-wrapper | 04-rewire-app-evm-to-state-memory | 5 |
| 06-remove-transitional-concrete-paths | 05-rewire-whirlpool-node-wrapper | 6 |

### Agent Dispatch

| Wave | Tasks | Parallel | Estimated Time |
|------|-------|----------|----------------|
| 1 | 01 | no | 5-10 min |
| 2 | 02 | no | 10-15 min |
| 3 | 03 | no | 20-30 min |
| 4 | 04 | no | 15-25 min |
| 5 | 05 | no | 5-10 min |
| 6 | 06 | no | 10-20 min |

## Task List

<!-- TASKS_START -->
- [ ] `[S]` [01-lock-interface-surface-in-state](tasks/01-lock-interface-surface-in-state.md) - Lock interface contracts in `state`
- [ ] `[M]` [02-scaffold-state-memory-crate](tasks/02-scaffold-state-memory-crate.md) ⚠️ DESTRUCTIVE - Add `state-memory` crate scaffold and workspace wiring
- [ ] `[L]` [03-move-concrete-db-and-revm-impls](tasks/03-move-concrete-db-and-revm-impls.md) - Move concrete DB types and revm impls to `state-memory`
- [ ] `[L]` [04-rewire-app-evm-to-state-memory](tasks/04-rewire-app-evm-to-state-memory.md) - Rewire `app-evm` concrete imports/deps
- [ ] `[S]` [05-rewire-whirlpool-node-wrapper](tasks/05-rewire-whirlpool-node-wrapper.md) - Rewire node wrapper to `state-memory`
- [ ] `[M]` [06-remove-transitional-concrete-paths](tasks/06-remove-transitional-concrete-paths.md) - Remove concrete re-exports from `state`
<!-- TASKS_END -->

## Artifact Registry

<!-- ARTIFACTS_START -->
| TestID | Planned Name | Planned Location | Actual Name | Actual Location | Status |
|--------|-------------|------------------|-------------|-----------------|--------|
| `TB-001` | state_interface_exports_compile | `crates/state/src/{lib.rs,traits.rs,error.rs}` | - | - | planned |
| `TB-002` | state_memory_scaffold_compile | `crates/state-memory/{Cargo.toml,src/lib.rs}` | - | - | planned |
| `TB-003` | state_memory_revm_impl_compile | `crates/state-memory/src/db.rs` | - | - | planned |
| `TB-004` | app_evm_concrete_import_rewire | `crates/app-evm/**` | - | - | planned |
| `TB-005` | whirlpool_node_teststatedb_rewire | `crates/whirlpool-node/**` | - | - | planned |
| `TB-006` | legacy_state_concrete_path_usage | workspace compile/doctest surfaces | - | - | planned |
| `TN-001` | state_interface_only_contract_compile | `crates/state/src/lib.rs` test module | - | - | planned |
| `TN-002` | state_error_internal_propagation | `crates/state/src/error.rs` test module | - | - | planned |
| `TN-003` | state_memory_inmemory_behavior_parity | `crates/state-memory/src/db.rs` tests | - | - | planned |
| `TN-004` | app_evm_uses_state_memory_imports | `crates/app-evm/tests/state_memory_imports.rs` | - | - | planned |
| `TN-005` | whirlpool_node_teststatedb_delegation | `crates/whirlpool-node/tests/test_state_db_delegation.rs` | - | - | planned |
| `TN-006` | state_concrete_paths_removed_guard | workspace-level compile/search gate harness | - | - | planned |
<!-- ARTIFACTS_END -->

## Final Verification

```bash
nix develop --command cargo check -p state -p state-memory -p app-evm -p whirlpool-node
nix develop --command cargo test -p state-memory --lib
nix develop --command cargo metadata --no-deps
```
