# TESTS

## Key Decisions

- **Grounded**: This step synthesizes test impact only; no migration ordering or design docs are modified.
- **Grounded**: Test contracts are aligned 1:1 with `MIGRATION.md` Step 1-6 so verification remains phase-ordered.
- **Grounded**: Existing crate tests remain the primary regression net; new tests are additive and target split-boundary failure modes.
- **[PROPOSED]**: Treat `TB-*` entries as migration-window breakage risks (including intermediate compile breaks) even if `main` is green.
- **[PROPOSED]**: Add explicit compile guards for canonical import paths and a post-cleanup guard to prevent concrete-path reintroduction into `state`.

## Broken Tests (Expected During Migration)

| TestID | Name | File | Reason | Fix Strategy | Migration Step |
|---|---|---|---|---|---|
| TB-001 | state_interface_exports_compile | `crates/state/src/{lib.rs,traits.rs,error.rs}` compile surfaces | Interface lock can accidentally drop `StateDb` / `StateError` visibility or marker wiring | Keep canonical exports in `state`; retain `DBErrorMarker` impl on `StateError` | Step 1 |
| TB-002 | state_memory_scaffold_compile | `crates/state-memory/Cargo.toml`, `crates/state-memory/src/lib.rs` | Missing workspace member/deps/re-exports while scaffolding new crate | Add crate wiring + `state-memory -> state` dependency + `state_memory::*` re-exports | Step 2 |
| TB-003 | state_memory_revm_impl_compile | `crates/state-memory/src/db.rs` | Moving `DatabaseRef`/`Database` impls can leave impl blocks unresolved | Move concrete types and impls together; keep signatures/semantics unchanged | Step 3 |
| TB-004 | app_evm_concrete_import_rewire | `crates/app-evm/**` | `state::InMemoryStateDb` import path removed from consumer | Rewire concrete imports to `state_memory::InMemoryStateDb`; preserve trait bounds on `state::traits::StateDb` | Step 4 |
| TB-005 | whirlpool_node_teststatedb_rewire | `crates/whirlpool-node/**` | Runtime wrapper (`TestStateDb`) still points to old concrete path | Switch wrapper concrete type to `state_memory::InMemoryStateDb` and keep `state::StateError` contract | Step 5 |
| TB-006 | legacy_state_concrete_path_usage | workspace compile/doctest surfaces | Step 6 removes `state` concrete re-exports, leaving stale imports | Sweep stale `state::{InMemoryStateDb,DbAccount}` usages and update docs/tests/examples | Step 6 |

## New Tests

| TestID | Name | Intent | File | Migration Step | Binding |
|---|---|---|---|---|---|
| TN-001 | state_interface_only_contract_compile | Ensure `state` exposes interface contract (`StateDb`, `StateError`) without requiring concrete crate symbols | `crates/state/src/lib.rs` test/compile module | Step 1 | No |
| TN-002 | state_error_internal_propagation | Add explicit coverage for `StateError::Internal` propagation and marker compatibility expectations | `crates/state/src/error.rs` test module (or `crates/state/tests/state_error_internal.rs`) | Step 1 | No |
| TN-003 | state_memory_inmemory_behavior_parity | Validate moved `InMemoryStateDb` preserves `state_root`, `commit`, and storage/read semantics | `crates/state-memory/src/db.rs` tests (or `crates/state-memory/tests/inmemory_parity.rs`) | Step 3 | No |
| TN-004 | app_evm_uses_state_memory_imports | Verify app-evm compiles/runs with `state_memory::InMemoryStateDb` concrete import and unchanged `StateDb` trait bounds | `crates/app-evm/tests/state_memory_imports.rs` | Step 4 | No |
| TN-005 | whirlpool_node_teststatedb_delegation | Add targeted smoke/contract test for `TestStateDb` delegation over `state_memory::InMemoryStateDb` | `crates/whirlpool-node/tests/test_state_db_delegation.rs` | Step 5 | No |
| TN-006 | state_concrete_paths_removed_guard | Add post-cleanup guard ensuring `state::{InMemoryStateDb,DbAccount}` is no longer referenced | workspace-level compile/search gate test harness | Step 6 | No |

## Test Contracts by Migration Step

| Migration Step | Contract(s) | Crates Primarily Affected |
|---|---|---|
| Step 1: Lock Interface Surface in `state` | `StateDb`/`StateError` remain canonical in `state`; `DBErrorMarker` impl remains attached to `StateError` | `state` |
| Step 2: Scaffold `state-memory` Crate | New implementation crate compiles independently and re-exports `DbAccount` + `InMemoryStateDb` | `state-memory` |
| Step 3: Move Concrete DB + `revm` Impl Blocks | Concrete behavior parity preserved after relocation; `DatabaseRef`/`Database` impl availability unchanged | `state-memory` |
| Step 4: Rewire `app-evm` to Concrete Crate | app-evm adopts `state_memory::InMemoryStateDb` while keeping interface trait usage from `state` | `app-evm`, `state` |
| Step 5: Rewire `whirlpool-node` Runtime Wrapper | `TestStateDb` wrapper delegates through `state-memory` concrete DB and retains `state::StateError` contract | `whirlpool-node`, `state-memory`, `state` |
| Step 6: Remove Transitional Concrete Paths from `state` | `state` becomes interface-only; no stale concrete imports remain in consumers/tests/docs | `state`, `state-memory`, `app-evm`, `whirlpool-node` |

## Per-Crate Test Changes Aligned to Migration

| Crate | Step Alignment | Test Changes |
|---|---|---|
| `state` | Steps 1, 6 | Add interface-only and `StateError::Internal` coverage; remove/guard concrete export assumptions |
| `state-memory` | Steps 2, 3 | Add/port in-memory DB parity tests plus `revm` impl compile coverage |
| `app-evm` | Step 4 | Update concrete import tests to `state_memory::InMemoryStateDb` while preserving `StateDb` trait-bound behavior |
| `whirlpool-node` | Step 5 | Add targeted `TestStateDb` delegation smoke/contract test for new concrete dependency path |
| workspace-wide | Step 6 | Add cleanup gate for stale concrete paths and run broad compile/test sweep |

## Verification Sequence

### Per-step verification commands

1. Step 1: `nix develop --command cargo check -p state`
2. Step 2: `nix develop --command cargo check -p state-memory && nix develop --command cargo metadata --no-deps`
3. Step 3: `nix develop --command cargo check -p state-memory && nix develop --command cargo test -p state-memory --lib`
4. Step 4: `nix develop --command cargo check -p app-evm && nix develop --command cargo test -p app-evm`
5. Step 5: `nix develop --command cargo check -p whirlpool-node`
6. Step 6: `nix develop --command cargo check -p state -p state-memory -p app-evm -p whirlpool-node && nix develop --command cargo test -p state-memory --lib`

### Confidence sweep

- `nix develop --command cargo test -p state`
- `nix develop --command cargo test -p app-evm`
- `nix develop --command cargo test -p whirlpool-node`
- `nix develop --command cargo test`

## Cross-reference Check

| Migration Step | Broken-test mapping (`TB-*`) | New-test/contract mapping (`TN-*`) | Status |
|---|---|---|---|
| Step 1 | TB-001 | TN-001, TN-002 | PASS |
| Step 2 | TB-002 | (Contract row: Step 2 crate scaffold contract) | PASS |
| Step 3 | TB-003 | TN-003 | PASS |
| Step 4 | TB-004 | TN-004 | PASS |
| Step 5 | TB-005 | TN-005 | PASS |
| Step 6 | TB-006 | TN-006 | PASS |

- All `MIGRATION.md` steps (1-6) are explicitly mapped to breakage and/or additive test contracts.
- Every `TB-*` entry includes an explicit fix strategy.
- No unmapped migration step detected; Step 5 synthesize gate is clear.
