# Migration Context (Explore Step 2b)

## Ordering Constraints

1. Preserve `state::traits::StateDb` and `state::StateError` exports first.
2. Introduce `state-memory` with concrete DB modules and `revm` impls next.
3. Update downstream concrete imports only after `state-memory` exports are stable.
4. Keep dependency direction acyclic (`state-memory -> state` only).
5. Remove temporary compatibility surfaces only after consumer compile/test stability.

## Recommended Migration Sequence

1. **Stabilize interface crate (`state`)**
   - Keep `traits` + `error` authoritative.
   - Ensure `StateError` remains public with `DBErrorMarker` impl.
   - Stop expanding concrete exports in `state`.

2. **Create implementation crate (`state-memory`)**
   - Move `DbAccount` and `InMemoryStateDb` from `state::db`.
   - Move `impl DatabaseRef for InMemoryStateDb` and `impl Database for InMemoryStateDb`.
   - Re-export `DbAccount` and `InMemoryStateDb` from crate root for consumer ergonomics.

3. **Rewire consumers**
   - `app-evm`: switch concrete DB imports to `state-memory` while keeping trait bounds on `state::traits::StateDb`.
   - `whirlpool-node`: switch `TestStateDb` wrapper to `state-memory::InMemoryStateDb`; keep `StateError` contract from `state`.

4. **Consolidate and clean up**
   - Remove stale concrete import paths from code/tests/docs.
   - Confirm no reintroduced reverse dependency.

## Suggested Batches

- **Batch A (low-medium risk)**: interface stabilization in `state` + create `state-memory` scaffolding.
- **Batch B (high risk)**: move concrete DB types and `revm` impls.
- **Batch C (medium-high risk)**: downstream consumer import rewrites and cleanup.

## High-Risk Files/Seams

- `crates/state/src/db.rs` (source of all concrete DB logic being extracted).
- `crates/app-evm/src/executor.rs` + `crates/app-evm/tests/*` (direct concrete DB usage).
- `crates/whirlpool-node/src/main.rs` (`TestStateDb` runtime bridge).

## Gating Checks During Migration

- Interface gate: `state::traits::StateDb` path remains stable.
- Implementation gate: `state-memory` exposes complete concrete API + `revm` impls.
- Dependency gate: no cycle and no `state -> state-memory` edge.
- Consumer gate: `app-evm` and `whirlpool-node` compile with new concrete paths.

## Source Artifacts

- `.design-scratch/refactor-e2e/split-state-interface-impl-20260305-0847/shared-dependency-graph.md`
- `.design-scratch/refactor-e2e/split-state-interface-impl-20260305-0847/shared-module-structure.md`
- `.design-scratch/refactor-e2e/split-state-interface-impl-20260305-0847/shared-impact-analysis.md`
