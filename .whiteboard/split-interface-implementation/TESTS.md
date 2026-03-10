# TESTS

## Key Decisions

- **Grounded**: Treat this phase as test-impact synthesis only; no source-code or migration-step edits are introduced here.
- **Grounded**: Test mapping follows `MIGRATION.md` Step 1-9 order so verification stays aligned with interface-first migration sequencing.
- **Grounded**: Existing crate suites are kept as primary safety net; targeted new tests are additive and focus on path compatibility plus trait-contract parity.
- **[PROPOSED]**: Represent expected breakage as "at-risk/broken during migration" (`TB-*`) even if mainline is currently green, because breakage is induced by intermediate refactor states.
- **[PROPOSED]**: Keep compatibility-window checks explicit (old and new trait paths both compile) until Step 9 cleanup removes legacy re-exports.

## Broken Tests (Expected During Migration)

| TestID | Name | File | Reason | Fix Strategy | Migration Step |
|---|---|---|---|---|---|
| TB-001 | consensus_app_trait_imports_compile | `crates/consensus/src/app.rs` tests/cfg blocks | `ConsensusApp` canonical path moves to `consensus::traits` | Update internal imports to canonical path; keep compatibility re-export until Step 9 | Step 1 |
| TB-002 | consensus_engine_trait_bounds_compile | `crates/consensus/src/engine.rs` tests/cfg blocks | `ConsensusEngine` trait surface centralized; old module-local assumptions can fail | Rebind trait imports to `consensus::traits::*` and preserve trait signatures unchanged | Step 1 |
| TB-003 | state_db_inmemory_compile_contract | `crates/state/src/db.rs` tests | New `StateDb` trait introduction can leave impl/tests disconnected | Add `StateDb for InMemoryStateDb` and assert `state_root`/`commit` contract in tests | Step 2 |
| TB-004 | app_evm_state_db_bound_compile | `crates/app-evm/src/executor.rs` tests and type checks | `StateDb` becomes trait boundary; concrete-type assumptions may fail | Move bounds to trait import (`state::traits::StateDb`) while preserving behavior | Step 2 |
| TB-005 | p2p_traits_interface_only_compile | `crates/p2p/src/traits.rs` tests/cfg feature checks | Concrete leakage in `traits.rs` can break interface-only invariants | Move impl-only items out of `traits.rs`; keep exports stable in `lib.rs` | Step 3 |
| TB-006 | app_txsource_path_compat_compile | `crates/app/src/traits.rs` and `crates/app/src/lib.rs` tests | `NoopTxSource`/`InMemoryTxPool` move out of `traits.rs` | Re-export moved types during transition; update local tests to `app::tx_source::*` | Step 4 |
| TB-007 | app_evm_state_provider_imports_compile | `crates/app-evm/src/executor.rs` tests | `StateProvider` moved into `app-evm::traits` | Keep compatibility re-export in `executor.rs`; migrate imports incrementally | Step 5 |
| TB-008 | simplex_commonware_block_bound_compile | `crates/consensus-simplex/src/{adapter.rs,engine.rs,tests.rs}` | `CommonwareBlock` moves from `types.rs` to `traits.rs`; generic bounds may mismatch | Move trait + blanket impl together; update imports in adapter/engine/tests to canonical path | Step 6 |
| TB-009 | p2p_commonware_transport_type_alignment | `crates/p2p-commonware/src/{provider.rs,tests.rs}` | New `CommonwareTransport` contract can diverge from existing sender/receiver wiring | Add trait impls on existing transport/provider types; verify send/recv parity tests | Step 7 |
| TB-010 | node_canonical_imports_compile | `crates/whirlpool-node/**` | Node still importing legacy paths after upstream trait moves | Update to canonical imports (`consensus::traits`, `app-evm::traits`, etc.) | Step 8 |
| TB-012 | legacy_path_usage_after_cleanup | workspace doctests/integration compile surfaces | Step 9 removes compatibility exports; any stale import fails | Final sweep for legacy paths; update doctests/examples and test imports | Step 9 |

## New Tests

| TestID | Name | Intent | File | Migration Step | Binding |
|---|---|---|---|---|---|
| TN-001 | consensus_traits_dual_path_compile | Validate old and canonical consensus trait paths compile during compatibility window | `crates/consensus/src/lib.rs` (cfg compile test module) | Step 1 | No |
| TN-002 | state_db_trait_contract_inmemory | Verify `InMemoryStateDb` satisfies `StateDb` semantics (`state_root`, `commit`) | `crates/state/src/db.rs` or `crates/state/tests/state_db_contract.rs` | Step 2 | No |
| TN-003 | app_evm_uses_state_db_trait_bound | Verify app-evm compiles/runs with trait-bound DB rather than concrete-only assumptions | `crates/app-evm/tests/state_db_trait_bound.rs` | Step 2 | No |
| TN-004 | p2p_traits_no_concrete_items | Assert `p2p::traits` remains interface-only (no impl/helper concrete types) | `crates/p2p/src/traits.rs` (structure/compile test) | Step 3 | No |
| TN-005 | app_tx_source_dual_path_compile | Ensure `app::traits::{NoopTxSource,InMemoryTxPool}` compatibility path and `app::tx_source::*` canonical path both compile pre-cleanup | `crates/app/src/lib.rs` test module | Step 4 | No |
| TN-006 | state_provider_dual_path_compile | Ensure `app-evm::executor::StateProvider` compatibility path and `app-evm::traits::StateProvider` canonical path both compile pre-cleanup | `crates/app-evm/src/lib.rs` test module | Step 5 | No |
| TN-007 | simplex_commonware_block_dual_path_compile | Ensure old `types::CommonwareBlock` compatibility export and new `traits::CommonwareBlock` path compile pre-cleanup | `crates/consensus-simplex/src/tests.rs` | Step 6 | No |
| TN-008 | commonware_transport_contract_send_recv | Validate `CommonwareTransport` contract parity with existing provider/sender/receiver behavior | `crates/p2p-commonware/src/tests.rs` | Step 7 | No |
| TN-009 | node_imports_canonical_only_guard | Ensure top-level nodes compile with canonical paths and do not add new legacy imports | `crates/whirlpool-node/tests/canonical_imports.rs` (or compile-check harness) | Step 8 | No |
| TN-010 | legacy_paths_fail_after_cleanup | Negative compile check (or lint-like search gate) ensuring removed compatibility paths are not referenced after Step 9 | workspace-level check script/test harness | Step 9 | No |

## Test Contracts by Migration Step

| Migration Step | Contract(s) | Crates Primarily Affected |
|---|---|---|
| Step 1: Consensus Trait Boundary Normalization | Trait contract parity for `ConsensusApp`, `Block`, `EventSink`, `ConsensusEngine`; dual-path compatibility until cleanup | `consensus` |
| Step 2: State Interface Introduction | New-trait introduction contract for `StateDb`; no behavior change for `InMemoryStateDb`; downstream trait-bound compatibility | `state`, `app-evm` |
| Step 3: P2P Contract Stabilization | Interface-only boundary contract in `p2p::traits`; feature/cfg compile stability | `p2p`, `p2p-commonware` |
| Step 4: App Interface/Implementation Split | Behavioral parity for moved concrete tx sources; dual-path compile compatibility | `app`, `app-evm` |
| Step 5: App-EVM StateProvider Relocation | Trait relocation parity (`StateProvider` signatures/associated types unchanged); EVM propose/verify flow unchanged | `app-evm`, `whirlpool-node` |
| Step 6: Consensus-Simplex CommonwareBlock Relocation | Adapter trait-bound integrity and blanket-impl parity across new trait module | `consensus-simplex` |
| Step 7: P2P-Commonware Transport Interface Introduction | New `CommonwareTransport` contract parity with existing send/recv/channel behavior | `p2p-commonware`, `consensus-simplex` |
| Step 8: Consumer Import Migration | Canonical import adoption in node consumers; no regressions from shimmed paths | `whirlpool-node` |
| Step 9: Compatibility Export Cleanup | Post-cleanup canonical-only contract; no legacy-path references remain in tests/docs/examples | workspace-wide |

## Per-Crate Test Changes Aligned to Migration

| Crate | Step Alignment | Test Changes |
|---|---|---|
| `consensus` | Step 1 | Add/adjust compile tests for canonical `consensus::traits` imports plus temporary dual-path compatibility checks |
| `state` | Step 2 | Add `StateDb` conformance tests for `InMemoryStateDb` (`state_root`, `commit`) |
| `p2p` | Step 3 | Add structure/compile guard ensuring `traits.rs` remains interface-only and feature-gated tests still compile |
| `app` | Step 4 | Move tx-source path expectations to `app::tx_source`; retain transition tests for old path re-exports |
| `app-evm` | Steps 2, 5 | Add trait-bound compile/behavior tests for `StateDb` and dual-path `StateProvider` imports |
| `consensus-simplex` | Step 6 | Update adapter/engine tests to canonical `traits::CommonwareBlock`; keep temporary compatibility-path compile checks |
| `p2p-commonware` | Step 7 | Add transport contract tests validating sender/receiver/provider behavior equivalence under `CommonwareTransport` |
| `whirlpool-node` | Step 8 | Add compile-use checks for canonical imports; remove any newly introduced legacy imports |
| workspace-wide | Step 9 | Add post-cleanup gate to fail on legacy trait-path usage in tests/examples/doctests |

## Verification Sequence

### Per-step verification commands

1. Step 1: `cargo check -p consensus && cargo test -p consensus --features mock`
2. Step 2: `cargo check -p state && cargo check -p app-evm && cargo test -p state`
3. Step 3: `cargo check -p p2p && cargo check -p p2p-commonware && cargo test -p p2p`
4. Step 4: `cargo check -p app && cargo check -p app-evm && cargo test -p app`
5. Step 5: `cargo check -p app-evm && cargo check -p whirlpool-node && cargo test -p app-evm`
6. Step 6: `cargo check -p consensus-simplex && cargo test -p consensus-simplex --lib`
7. Step 7: `cargo check -p p2p-commonware && cargo check -p consensus-simplex && cargo test -p p2p-commonware`
8. Step 8: `cargo check -p whirlpool-node && cargo test -p whirlpool-node`
9. Step 9: `cargo check --workspace && cargo test --workspace`

### Stability fallback

- If `p2p-commonware` test/build intermittently fails due to known resource/ICE instability, rerun with `CARGO_BUILD_JOBS=1 cargo test -p p2p-commonware`.

## Cross-reference Check

- All migration steps (1-9) are mapped to at least one broken-test entry (`TB-*`) and at least one contract or new-test entry (`TN-*`).
- Every broken-test row includes an explicit fix strategy.
- No unmapped migration step detected; therefore no Step-5 blocker is introduced.
