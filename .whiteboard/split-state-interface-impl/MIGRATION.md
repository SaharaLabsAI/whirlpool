# MIGRATION

## Key Decisions

- **Grounded**: Keep `StateDb`, `StateError`, and `DBErrorMarker` in `state` so interface-only consumers keep a stable contract path.
- **Grounded**: Move concrete storage and `revm` database impls together into `state-memory` to avoid split ownership of one runtime behavior.
- **Assumption**: Workspace concrete consumers are limited to `app-evm` and `whirlpool-node`; migration rewires these first-class call sites in the same wave.
- **Rejected alternative**: Keep re-exporting `InMemoryStateDb` from `state` long-term. Rejected because it preserves interface/implementation coupling and encourages a reverse dependency.
- **Rationale**: Ordered additive steps with per-step compile gates preserve incremental compilability and reduce high-risk churn to import/dependency rewiring.

## Step 1: Lock Interface Surface in `state`

- **Scope**: Ensure `state` is authoritative for interface contract symbols.
- **Prerequisite**: Current workspace baseline is green.
- **Changes**:
  - [ ] Keep `state::traits::StateDb` canonical and public.
  - [ ] Keep `state::error::StateError` public and retain `DBErrorMarker` impl in `state`.
  - [ ] Stop adding new concrete exports from `state`.
- **Verification**:
  - `nix develop --command cargo check -p state`
- **Rollback**:
  - Revert only interface-surface edits in `crates/state/src/lib.rs`, `crates/state/src/traits.rs`, and `crates/state/src/error.rs`.
  - Re-run `nix develop --command cargo check -p state`.

## Step 2: Scaffold `state-memory` Crate

- **Scope**: Introduce new implementation crate with stable public entrypoints.
- **Prerequisite**: Step 1 complete.
- **Changes**:
  - [ ] Add `crates/state-memory/Cargo.toml` and `crates/state-memory/src/lib.rs`.
  - [ ] Add `state-memory -> state` dependency and required `revm`/`alloy-genesis` deps.
  - [ ] Re-export `DbAccount` and `InMemoryStateDb` at `state_memory::*`.
- **Verification**:
  - `nix develop --command cargo check -p state-memory`
  - `nix develop --command cargo metadata --no-deps`
- **Rollback**:
  - Remove `state-memory` crate scaffolding and workspace member entry.
  - Re-run `nix develop --command cargo check -p state`.

## Step 3: Move Concrete DB + `revm` Impl Blocks

- **Scope**: Relocate concrete types and database trait impls into `state-memory` without semantic changes.
- **Prerequisite**: Step 2 complete and `state-memory` compiles.
- **Changes**:
  - [ ] Move `DbAccount` and `InMemoryStateDb` from `state::db` to `state_memory::db`.
  - [ ] Move `impl DatabaseRef for InMemoryStateDb` and `impl Database for InMemoryStateDb` to `state-memory`.
  - [ ] Keep behavior identical (constructors, state root algorithm, commit semantics).
- **Verification**:
  - `nix develop --command cargo check -p state-memory`
  - `nix develop --command cargo test -p state-memory --lib`
- **Rollback**:
  - Restore moved concrete code to `crates/state/src/db.rs`.
  - Re-run `nix develop --command cargo check -p state -p state-memory`.

## Step 4: Rewire `app-evm` to Concrete Crate

- **Scope**: Move concrete DB imports to `state-memory` while preserving interface traits from `state`.
- **Prerequisite**: Step 3 complete and concrete exports stable.
- **Changes**:
  - [ ] Replace `state::InMemoryStateDb` imports with `state_memory::InMemoryStateDb` in runtime and tests.
  - [ ] Keep trait bounds on `state::traits::StateDb` unchanged.
  - [ ] Add/update `state-memory` dependency in `crates/app-evm/Cargo.toml`.
- **Verification**:
  - `nix develop --command cargo check -p app-evm`
  - `nix develop --command cargo test -p app-evm`
- **Rollback**:
  - Revert `app-evm` import/dependency changes only.
  - Re-run `nix develop --command cargo check -p app-evm`.

## Step 5: Rewire `whirlpool-node` Runtime Wrapper

- **Scope**: Switch node runtime wrapper to concrete DB from `state-memory`.
- **Prerequisite**: Step 4 complete.
- **Changes**:
  - [ ] Update `TestStateDb` wrapper to use `state_memory::InMemoryStateDb`.
  - [ ] Keep error contract on `state::StateError`.
  - [ ] Add/update `state-memory` dependency in `crates/whirlpool-node/Cargo.toml`.
- **Verification**:
  - `nix develop --command cargo check -p whirlpool-node`
- **Rollback**:
  - Revert node import/dependency edits only.
  - Re-run `nix develop --command cargo check -p whirlpool-node`.

## Step 6: Remove Transitional Concrete Paths from `state`

- **Scope**: Finalize interface/implementation separation and remove stale paths.
- **Prerequisite**: Steps 1-5 complete and all targeted consumers compile.
- **Changes**:
  - [ ] Remove concrete re-exports (`DbAccount`, `InMemoryStateDb`) from `state`.
  - [ ] Keep only interface/shared exports (`StateDb`, `StateError`) in `state`.
  - [ ] Update docs/comments referencing old concrete paths.
- **Verification**:
  - `nix develop --command cargo check -p state -p state-memory -p app-evm -p whirlpool-node`
  - `nix develop --command cargo test -p state-memory --lib`
- **Rollback**:
  - Reintroduce removed concrete re-exports in `state` as a bounded compatibility patch.
  - Re-run `nix develop --command cargo check -p state -p app-evm -p whirlpool-node`.

## Compilability Invariant Review

- Interface contract (`state`) is stabilized before concrete-symbol movement.
- New implementation crate (`state-memory`) compiles before downstream rewiring starts.
- Consumer rewiring is ordered (`app-evm` then `whirlpool-node`) after concrete exports exist.
- Dependency direction remains one-way (`state-memory -> state`), with explicit cycle gate via `cargo metadata`.
- Each step includes independent verification and bounded rollback to preserve incremental compilability.
