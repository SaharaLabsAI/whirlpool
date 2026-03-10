# MIGRATION

## Key Decisions

- **Grounded**: Migration order follows the established dependency chain: foundation (`consensus`, `p2p`, `state`) -> app abstraction (`app`) -> adapters (`app-evm`, `consensus-simplex`, `p2p-commonware`) -> top-level consumers (`whirlpool-node`).
- **Grounded**: Every symbol move is staged with compatibility re-exports before downstream import rewrites.
- **[PROPOSED]**: Keep final compatibility-export removal as a dedicated final step so all prior steps preserve dual-path compilation.
- **[PROPOSED]**: Use crate-targeted `cargo check -p <crate>` verification at each step, plus focused consumer checks in cleanup.

## Step 1: Consensus Trait Boundary Normalization (Foundation)

- **Scope**: Create explicit `consensus::traits` interface boundary without changing existing behavior.
- **Prerequisite**: Baseline compiles on current mainline.
- **Changes**:
  - [ ] Add `crates/consensus/src/traits.rs` that re-exports or defines canonical trait surface for `ConsensusApp`, `Block`, `EventSink`, and `ConsensusEngine`.
  - [ ] Update `crates/consensus/src/lib.rs` to expose `pub mod traits;` and re-export canonical symbols from `traits`.
  - [ ] Keep compatibility exports from `app.rs`, `block.rs`, `event.rs`, and `engine.rs` so old paths remain valid.
- **Verification**:
  - `cargo check -p consensus`
- **Rollback**:
  - Revert changes in `crates/consensus/src/lib.rs` and remove `crates/consensus/src/traits.rs`.
  - Re-run `cargo check -p consensus`.

## Step 2: State Interface Introduction (Foundation)

- **Scope**: Introduce `state::traits::StateDb` as additive interface over current concrete DB.
- **Prerequisite**: Step 1 complete and `consensus` compiling.
- **Changes**:
  - [ ] Add `crates/state/src/traits.rs` with `StateDb` trait containing `state_root` and `commit` contract.
  - [ ] Implement `StateDb` for `InMemoryStateDb` in `crates/state/src/db.rs` or an impl-focused module.
  - [ ] Update `crates/state/src/lib.rs` to expose `pub mod traits;` and `pub use traits::StateDb;` while preserving existing concrete exports.
- **Verification**:
  - `cargo check -p state`
  - `cargo check -p app-evm`
- **Rollback**:
  - Remove `traits.rs`, remove `StateDb` export, and remove temporary impl.
  - Re-run `cargo check -p state -p app-evm`.

## Step 3: P2P Contract Stabilization (Foundation)

- **Scope**: Keep `p2p::traits` as interface-only canonical surface and ensure no concrete leakage.
- **Prerequisite**: Steps 1-2 complete.
- **Changes**:
  - [ ] Confirm `crates/p2p/src/traits.rs` remains interface-only and contains `PeerId`, `NetworkSender`, `NetworkReceiver`, `NetworkProvider`.
  - [ ] If needed, move any implementation-oriented items from `traits.rs` into implementation modules.
  - [ ] Keep `crates/p2p/src/lib.rs` exports stable to avoid downstream churn during adapter migration.
- **Verification**:
  - `cargo check -p p2p`
  - `cargo check -p p2p-commonware`
- **Rollback**:
  - Restore previous `p2p` module/export layout and re-run checks.

## Step 4: App Interface/Implementation Split

- **Scope**: Make `app::traits` interface-only by relocating concrete tx-source types.
- **Prerequisite**: Foundation steps complete and compiling.
- **Changes**:
  - [ ] Add `crates/app/src/tx_source.rs` containing `NoopTxSource` and `InMemoryTxPool` implementations.
  - [ ] Remove concrete tx-source definitions from `crates/app/src/traits.rs`, leaving only `Application` and `TxSource`.
  - [ ] Update `crates/app/src/lib.rs` with `pub mod tx_source;` plus compatibility re-exports so old imports continue compiling.
  - [ ] Update in-crate tests for moved module paths (keep external behavior unchanged).
- **Verification**:
  - `cargo check -p app`
  - `cargo check -p app-evm`
- **Rollback**:
  - Move tx-source types back into `traits.rs`, remove `tx_source.rs`, restore old exports.
  - Re-run `cargo check -p app -p app-evm`.

## Step 5: App-EVM StateProvider Relocation (Adapter)

- **Scope**: Move `StateProvider` trait from executor implementation to interface module.
- **Prerequisite**: Step 4 complete (app split stable).
- **Changes**:
  - [ ] Add `crates/app-evm/src/traits.rs` and move `StateProvider` trait there.
  - [ ] Update `crates/app-evm/src/lib.rs` to expose `pub mod traits;` and `pub use traits::StateProvider;`.
  - [ ] Keep compatibility re-export in `crates/app-evm/src/executor.rs` during transition.
  - [ ] Keep `EvmApplication<DB: StateProvider + ...>` bounds unchanged; only import paths move.
- **Verification**:
  - `cargo check -p app-evm`
  - `cargo check -p whirlpool-node`
- **Rollback**:
  - Reintroduce trait in `executor.rs`, remove `traits.rs`, restore prior exports/imports.
  - Re-run `cargo check -p app-evm -p whirlpool-node`.

## Step 6: Consensus-Simplex CommonwareBlock Relocation (Adapter, High Risk)

- **Scope**: Move `CommonwareBlock` from `types.rs` to dedicated interface module.
- **Prerequisite**: Steps 1-5 complete and green.
- **Changes**:
  - [ ] Add `crates/consensus-simplex/src/traits.rs` and move `CommonwareBlock` trait + blanket impl there.
  - [ ] Update `crates/consensus-simplex/src/lib.rs` to expose canonical `traits` path and keep compatibility export from `types`.
  - [ ] Update internal imports in `adapter.rs`, `engine.rs`, and tests to use canonical trait path.
- **Verification**:
  - `cargo check -p consensus-simplex`
  - `cargo test -p consensus-simplex --lib`
- **Rollback**:
  - Move `CommonwareBlock` back to `types.rs`, restore old imports, keep interface module removal isolated to this step.
  - Re-run `cargo check -p consensus-simplex`.

## Step 7: P2P-Commonware Transport Interface Introduction (Adapter, High Risk)

- **Scope**: Introduce explicit transport contract separate from provider/sender/receiver implementations.
- **Prerequisite**: Step 3 and Step 6 complete.
- **Changes**:
  - [ ] Add `crates/p2p-commonware/src/traits.rs` introducing `CommonwareTransport` as additive contract.
  - [ ] Implement `CommonwareTransport` for relevant transport/provider types in `provider.rs` or focused impl module.
  - [ ] Update `crates/p2p-commonware/src/lib.rs` to export `traits` and keep current public constructors/exports stable.
- **Verification**:
  - `cargo check -p p2p-commonware`
  - `cargo check -p consensus-simplex`
- **Rollback**:
  - Remove `traits.rs` and associated impls, restore previous provider-focused API.
  - Re-run `cargo check -p p2p-commonware -p consensus-simplex`.

## Step 8: Consumer Import Migration (Nodes)

- **Scope**: Move consumers to canonical interface paths once all compatibility shims exist.
- **Prerequisite**: Steps 1-7 complete with dual-path compatibility intact.
- **Changes**:
  - [ ] Update `whirlpool-node` imports to canonical trait paths (`consensus::traits`, `app-evm::traits`, etc.).
  - [ ] Remove any newly introduced old-path imports in integration points.
- **Verification**:
  - `cargo check -p whirlpool-node`
- **Rollback**:
  - Revert node import edits only; keep upstream compatibility shims in place.
  - Re-run node checks.

## Step 9: Compatibility Export Cleanup (Final)

- **Scope**: Remove transitional re-exports only after all crates compile on canonical paths.
- **Prerequisite**: Step 8 complete and all checks green.
- **Changes**:
  - [ ] Remove compatibility exports in `consensus`, `app`, `consensus-simplex`, and `app-evm` that preserve old trait paths.
  - [ ] Ensure crate-root exports reference canonical interface modules.
  - [ ] Update docs/comments referencing legacy paths.
- **Verification**:
  - `cargo check --workspace`
  - `cargo test --workspace`
- **Rollback**:
  - Reintroduce removed compatibility re-exports as a single patch.
  - Re-run `cargo check --workspace`.

## Compilability Invariant Review

- Interface modules are introduced before symbol relocation in every crate.
- Compatibility re-exports remain in place until consumers are migrated.
- High-risk generic-bound migrations (`consensus-simplex`, `p2p-commonware`, `app-evm`) occur after foundation stabilization.
- Every step has independent verification and bounded rollback to preserve incremental compilability.
