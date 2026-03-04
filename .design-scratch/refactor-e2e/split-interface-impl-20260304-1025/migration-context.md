# Migration Context (Explore Step 2b)

## Compilation Order Constraints

1. Keep public re-exports stable before moving definitions.
2. Introduce new trait modules first, then move/bridge concrete impl modules.
3. Update downstream imports only after upstream crates expose compatibility re-exports.
4. Preserve trait bounds used by generic adapters (`consensus-simplex`, `p2p-commonware`, `app-evm`) until all dependents compile.
5. Avoid introducing any reverse dependency (`state`/`p2p`/`consensus` must remain foundational).

## Dependency Chain (Structural)

- Foundation interfaces: `consensus`, `p2p`, `state`
- App abstraction: `app`
- Adapters/implementations: `consensus-simplex`, `p2p-commonware`, `app-evm`
- Top-level consumers: `whirlpool-node`, `whirlpool-node-simple`

Recommended migration direction: foundation -> app -> adapter crates -> top-level nodes.

## Ordered Migration Steps

1. **Foundation traits normalization**
   - Add `consensus::traits` module re-exporting `ConsensusApp`, `Block`, `EventSink`, `ConsensusEngine`.
   - Keep existing paths (`consensus::app`, `consensus::block`, `consensus::event`, `consensus::engine`) as compatibility exports.

2. **State interface introduction**
   - Introduce `state::traits::StateDb` and implement it for `InMemoryStateDb`.
   - Re-export `StateDb` from `state::lib` while preserving existing concrete exports.

3. **App split completion**
   - Keep `Application` and `TxSource` in interface module.
   - Move `NoopTxSource` and `InMemoryTxPool` into `app::tx_source` and re-export at crate root to avoid breakage.

4. **P2P adapter-safe stabilization**
   - Keep `p2p::traits` contract stable (no path churn) while preparing optional interface-only re-export module.
   - Validate `p2p-commonware` impl blocks still satisfy unchanged trait bounds.

5. **consensus-simplex trait relocation**
   - Move `CommonwareBlock` to `consensus-simplex::traits` and maintain compatibility re-export from prior location.
   - Update internal uses first (`adapter.rs`, `engine.rs`, tests), then external imports.

6. **app-evm trait relocation**
   - Move `StateProvider` to `app-evm::traits` with compatibility export in `executor` during transition.
   - Keep `EvmApplication<DB: StateProvider + ...>` bound unchanged while imports migrate.

7. **p2p-commonware transport interface**
   - Introduce `CommonwareTransport` trait in `p2p-commonware::traits` to separate transport contract from provider/receiver/sender implementations.
   - Stage adoption behind re-exports to keep compile graph acyclic.

8. **Consumer cleanup**
   - Update `whirlpool-node` and `whirlpool-node-simple` imports to canonical interface paths only.
   - Remove temporary compatibility exports once all crates compile and tests pass.

## Suggested Batching

- **Batch A (low risk)**: `consensus` trait module consolidation + `state::traits::StateDb` introduction.
- **Batch B (medium risk)**: `app` concrete tx source move + `app-evm::StateProvider` move with compatibility shims.
- **Batch C (high risk)**: `consensus-simplex::CommonwareBlock` move + `p2p-commonware::CommonwareTransport` introduction + downstream import cleanup.

## High-Risk Areas

- Generic bounds in `consensus-simplex/src/engine.rs` and `consensus-simplex/src/adapter.rs` (multi-trait constraints).
- `p2p-commonware/src/provider.rs` + `lib.rs` multiplex implementations tied to `NetworkProvider` associated types.
- `app-evm/src/executor.rs` where `StateProvider` trait and `EvmApplication` implementation are currently co-located.
- Cross-crate import surfaces in node binaries (`whirlpool-node/src/main.rs`, `whirlpool-node-simple/src/main.rs`).

## Raw Data Pointers

- `.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-dependency-graph.md`
- `.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-module-structure.md`
- `.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-test-coverage.md`
- `docs/refactor/split-interface-implementation/INTENT.md`
