# Migration Step Map

## Task Entries

### 01-lock-interface-surface-in-state
- migration_step: 1
- description: Lock and verify interface contracts in `state` (`StateDb`, `StateError`, `DBErrorMarker`) while avoiding new concrete exports.
- affected_crates: [state]
- target_files:
  - crates/state/src/lib.rs
  - crates/state/src/traits.rs
  - crates/state/src/error.rs
- complexity: S
- dependencies: none
- wave: 1
- change_type: restructure
- test_contracts: [TB-001, TN-001, TN-002]
- rollback:
  - Revert only interface-surface edits in `crates/state/src/lib.rs`, `crates/state/src/traits.rs`, and `crates/state/src/error.rs`.
  - Re-run `nix develop --command cargo check -p state`.

### 02-scaffold-state-memory-crate
- migration_step: 2
- description: Add `state-memory` crate scaffold, workspace wiring, and root re-exports for concrete types.
- affected_crates: [state-memory, workspace]
- target_files:
  - Cargo.toml
  - crates/state-memory/Cargo.toml
  - crates/state-memory/src/lib.rs
- complexity: M
- dependencies: [01-lock-interface-surface-in-state]
- wave: 2
- change_type: move
- test_contracts: [TB-002]
- rollback:
  - Remove `state-memory` crate scaffolding and workspace member entry.
  - Re-run `nix develop --command cargo check -p state`.

### 03-move-concrete-db-and-revm-impls
- migration_step: 3
- description: Relocate `DbAccount`, `InMemoryStateDb`, and `DatabaseRef`/`Database` impls from `state` to `state-memory` with behavior parity.
- affected_crates: [state, state-memory]
- target_files:
  - crates/state/src/db.rs
  - crates/state/src/lib.rs
  - crates/state-memory/src/db.rs
  - crates/state-memory/src/lib.rs
- complexity: L
- dependencies: [02-scaffold-state-memory-crate]
- wave: 3
- change_type: move
- test_contracts: [TB-003, TN-003]
- rollback:
  - Restore moved concrete code to `crates/state/src/db.rs`.
  - Re-run `nix develop --command cargo check -p state -p state-memory`.

### 04-rewire-app-evm-to-state-memory
- migration_step: 4
- description: Update `app-evm` concrete DB imports and dependency to `state-memory`, while preserving `state::traits::StateDb` bounds.
- affected_crates: [app-evm, state-memory, state]
- target_files:
  - crates/app-evm/Cargo.toml
  - crates/app-evm/src/executor.rs
  - crates/app-evm/tests/application_integration.rs
  - crates/app-evm/tests/cross_crate_flows.rs
  - crates/app-evm/tests/evm_execution_integration.rs
  - crates/app-evm/tests/integration.rs
- complexity: L
- dependencies: [03-move-concrete-db-and-revm-impls]
- wave: 4
- change_type: move
- test_contracts: [TB-004, TN-004]
- rollback:
  - Revert `app-evm` import/dependency changes only.
  - Re-run `nix develop --command cargo check -p app-evm`.

### 05-rewire-whirlpool-node-wrapper
- migration_step: 5
- description: Switch `whirlpool-node` wrapper concrete DB type to `state_memory::InMemoryStateDb` and keep `state::StateError` contract.
- affected_crates: [whirlpool-node, state-memory, state]
- target_files:
  - crates/whirlpool-node/Cargo.toml
  - crates/whirlpool-node/src/main.rs
- complexity: S
- dependencies: [04-rewire-app-evm-to-state-memory]
- wave: 5
- change_type: move
- test_contracts: [TB-005, TN-005]
- rollback:
  - Revert node import/dependency edits only.
  - Re-run `nix develop --command cargo check -p whirlpool-node`.

### 06-remove-transitional-concrete-paths
- migration_step: 6
- description: Remove concrete re-exports from `state`, keep interface-only exports, and update stale references/docs/comments.
- affected_crates: [state, state-memory, app-evm, whirlpool-node]
- target_files:
  - crates/state/src/lib.rs
  - crates/state/src/db.rs
  - crates/app-evm/**
  - crates/whirlpool-node/**
- complexity: M
- dependencies: [05-rewire-whirlpool-node-wrapper]
- wave: 6
- change_type: delete
- test_contracts: [TB-006, TN-006]
- rollback:
  - Reintroduce removed concrete re-exports in `state` as a bounded compatibility patch.
  - Re-run `nix develop --command cargo check -p state -p app-evm -p whirlpool-node`.

## Batched Step Handling
- No batched steps explicitly marked in MIGRATION.md.

## Ordering Verification (pre-gate)
- Tasks are strictly ordered by migration steps 1 -> 6.
- Each task depends on the immediate prior task to preserve compilability invariant.
