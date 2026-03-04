# split-interface-from-implementation - Execution Plan

## TL;DR

| Field | Value |
|-------|-------|
| **Summary** | Split trait/interface definitions from concrete implementations across foundation, app, and adapter crates while preserving compatibility until final cleanup |
| **Deliverables** | Canonical `traits` modules, migrated imports in consumers, compatibility-export cleanup, rollback-ready tasks |
| **Effort** | 9 tasks, 3 waves, estimated L complexity |
| **Critical path** | `01` -> `02` -> `03` -> `04` -> `05` -> `06` -> `07` -> `08` -> `09` |
| **Rollback** | Each task has scoped rollback; full rollback proceeds reverse task order |

## Context

**Design source**: `docs/refactor/split-interface-implementation/`
**Migration step source**: `docs/refactor/split-interface-implementation/MIGRATION.md` (9 steps)

### Refactor Contract Table (excerpt)

| Crate | Role | Change Type | Risk Level |
|-------|------|------------|------------|
| `consensus` | source/target | move (trait canonicalization) | high |
| `state` | target | introduce interface trait | medium |
| `p2p` | both | interface-only stabilization | low |
| `app` | both | move concrete types out of traits module | medium |
| `app-evm` | both | move trait from executor to traits module | medium-high |
| `consensus-simplex` | both | move generic-bound trait to traits module | high |
| `p2p-commonware` | target | introduce transport trait | high |
| `whirlpool-node` | consumer | import migration | medium |
| `whirlpool-node-simple` | consumer | import migration | medium |

## Work Objectives

### Definition of Done

```bash
cargo check --workspace
cargo test --workspace
```

### Must NOT Have

- No edits under `vendor/` or `third_party/`
- No semantic trait/API changes beyond path/module relocation
- No compatibility-export cleanup before consumer import migration completes

## Verification Strategy

- Evidence logs per task: `.sisyphus/plans/split-interface-from-implementation/evidence/NN-<slug>.log`
- Pre/Post gates enforce incremental compilability and test coverage mapping
- Rollback sections are mandatory and include task dependency notes

## Execution Strategy

### Dependency Matrix

| Task | Depends On | Wave |
|------|------------|------|
| 01-consensus-traits-boundary | none | 1 |
| 02-state-statedb-introduction | 01-consensus-traits-boundary | 1 |
| 03-p2p-traits-stabilization | 02-state-statedb-introduction | 1 |
| 04-app-txsource-split | 03-p2p-traits-stabilization | 2 |
| 05-app-evm-stateprovider-relocation | 04-app-txsource-split | 2 |
| 06-consensus-simplex-commonwareblock-relocation | 05-app-evm-stateprovider-relocation | 3 |
| 07-p2p-commonware-transport-introduction | 06-consensus-simplex-commonwareblock-relocation | 3 |
| 08-node-consumer-import-migration | 07-p2p-commonware-transport-introduction | 3 |
| 09-compatibility-export-cleanup | 08-node-consumer-import-migration | 3 |

### Agent Dispatch

| Wave | Tasks | Parallel | Estimated Time |
|------|-------|----------|----------------|
| 1 | 01, 02, 03 | no (ordered foundation) | 20-30 min |
| 2 | 04, 05 | no (ordered app boundary) | 20-30 min |
| 3 | 06, 07, 08, 09 | no (high-risk adapters + cleanup) | 35-50 min |

## Task List

<!-- TASKS_START -->
- [ ] `[M]` [01-consensus-traits-boundary](tasks/01-consensus-traits-boundary.md) - Canonicalize consensus traits in `consensus::traits` with compatibility exports
- [ ] `[M]` [02-state-statedb-introduction](tasks/02-state-statedb-introduction.md) - Introduce `state::traits::StateDb` and wire `InMemoryStateDb`
- [ ] `[S]` [03-p2p-traits-stabilization](tasks/03-p2p-traits-stabilization.md) - Enforce interface-only `p2p::traits` boundary
- [ ] `[M]` [04-app-txsource-split](tasks/04-app-txsource-split.md) - Move concrete tx sources from `app::traits` to `app::tx_source`
- [ ] `[M]` [05-app-evm-stateprovider-relocation](tasks/05-app-evm-stateprovider-relocation.md) - Move `StateProvider` to `app-evm::traits`
- [ ] `[L]` [06-consensus-simplex-commonwareblock-relocation](tasks/06-consensus-simplex-commonwareblock-relocation.md) - Relocate `CommonwareBlock` to canonical traits module
- [ ] `[M]` [07-p2p-commonware-transport-introduction](tasks/07-p2p-commonware-transport-introduction.md) - Introduce additive `CommonwareTransport` contract
- [ ] `[M]` [08-node-consumer-import-migration](tasks/08-node-consumer-import-migration.md) - Migrate node consumers to canonical trait import paths
- [ ] `[M]` [09-compatibility-export-cleanup](tasks/09-compatibility-export-cleanup.md) - Remove transitional compatibility re-exports and legacy path references
<!-- TASKS_END -->

## Artifact Registry

<!-- ARTIFACTS_START -->
| TestID | Planned Name | Planned Location | Actual Name | Actual Location | Status |
|--------|-------------|------------------|-------------|-----------------|--------|
| `TB-001` | consensus_app_trait_imports_compile | `crates/consensus/src/app.rs` test blocks | - | - | planned |
| `TB-002` | consensus_engine_trait_bounds_compile | `crates/consensus/src/engine.rs` test blocks | - | - | planned |
| `TB-003` | state_db_inmemory_compile_contract | `crates/state/src/db.rs` tests | - | - | planned |
| `TB-004` | app_evm_state_db_bound_compile | `crates/app-evm/src/executor.rs` tests | - | - | planned |
| `TB-005` | p2p_traits_interface_only_compile | `crates/p2p/src/traits.rs` tests | - | - | planned |
| `TB-006` | app_txsource_path_compat_compile | `crates/app/src/{traits.rs,lib.rs}` tests | - | - | planned |
| `TB-007` | app_evm_state_provider_imports_compile | `crates/app-evm/src/executor.rs` tests | - | - | planned |
| `TB-008` | simplex_commonware_block_bound_compile | `crates/consensus-simplex/src/{adapter.rs,engine.rs,tests.rs}` | - | - | planned |
| `TB-009` | p2p_commonware_transport_type_alignment | `crates/p2p-commonware/src/{provider.rs,tests.rs}` | - | - | planned |
| `TB-010` | node_canonical_imports_compile | `crates/whirlpool-node/**` | - | - | planned |
| `TB-011` | node_simple_canonical_imports_compile | `crates/whirlpool-node-simple/**` | - | - | planned |
| `TB-012` | legacy_path_usage_after_cleanup | workspace compile/doctest surfaces | - | - | planned |
| `TN-001` | consensus_traits_dual_path_compile | `crates/consensus/src/lib.rs` cfg test module | - | - | planned |
| `TN-002` | state_db_trait_contract_inmemory | `crates/state/src/db.rs` or `crates/state/tests/state_db_contract.rs` | - | - | planned |
| `TN-003` | app_evm_uses_state_db_trait_bound | `crates/app-evm/tests/state_db_trait_bound.rs` | - | - | planned |
| `TN-004` | p2p_traits_no_concrete_items | `crates/p2p/src/traits.rs` structure/compile test | - | - | planned |
| `TN-005` | app_tx_source_dual_path_compile | `crates/app/src/lib.rs` test module | - | - | planned |
| `TN-006` | state_provider_dual_path_compile | `crates/app-evm/src/lib.rs` test module | - | - | planned |
| `TN-007` | simplex_commonware_block_dual_path_compile | `crates/consensus-simplex/src/tests.rs` | - | - | planned |
| `TN-008` | commonware_transport_contract_send_recv | `crates/p2p-commonware/src/tests.rs` | - | - | planned |
| `TN-009` | node_imports_canonical_only_guard | `crates/whirlpool-node/tests/canonical_imports.rs` | - | - | planned |
| `TN-010` | legacy_paths_fail_after_cleanup | workspace-level compile/lint harness | - | - | planned |
<!-- ARTIFACTS_END -->

## Final Verification

```bash
cargo check --workspace && cargo test --workspace && echo "PLAN COMPLETE"
```
