# Impact Context (Explore Step 2b)

## Blast Radius Summary

| Area | Symbols | Direct crates | Structural dependents | Impact |
| --- | --- | --- | --- | --- |
| Interface contract | `StateDb`, `StateError`, `DBErrorMarker` | `state` | `app-evm`, `whirlpool-node` | Medium |
| Concrete DB implementation | `DbAccount`, `InMemoryStateDb`, `DatabaseRef impl`, `Database impl` | `state` -> `state-memory` (new) | `app-evm`, `whirlpool-node` | High |
| Consumer imports and wiring | `state::InMemoryStateDb` callsites | `app-evm`, `whirlpool-node` | tests/docs referencing concrete type | High |

## Primary Impact Findings

1. `StateDb` is already a stable interface in `state::traits`; keeping it in `state` preserves trait-bound continuity for interface-only consumers.
2. `StateError` and `DBErrorMarker` must remain in `state` so all `revm` database impls in the new implementation crate can continue using a shared error contract.
3. `InMemoryStateDb` and companion concrete symbols are the largest blast-radius move because they are currently consumed directly in runtime wiring and tests.
4. `DatabaseRef`/`Database` impl relocation is sensitive: missing trait impl visibility or incorrect error type linkage will break compile-time conformance.
5. Path churn is concentrated in concrete imports (`state::InMemoryStateDb` -> `state_memory::InMemoryStateDb`), while trait imports should remain stable.

## Cross-Crate Dependency Notes

- Target dependency shape: `state-memory -> state` (one-way).
- Forbidden shape: `state -> state-memory` (would violate interface-first split and risk cycles).
- Consumer pattern after split:
  - Interface-only crates depend on `state`.
  - Concrete runtime/test crates depend on both `state` and `state-memory`.

## Circularity Result

- Verification executed via `cargo metadata` graph traversal over workspace members.
- Result: `CIRCULAR_DEPENDENCY_CHECK: PASS` (no cycles detected).

## Risks and Unknowns

- **High risk**: missed concrete import updates in `app-evm` and `whirlpool-node`.
- **Medium risk**: incomplete re-export strategy during transition causing unresolved imports.
- **Unknown**: complete inventory of non-code references (docs/scripts/examples) that mention old concrete paths.

## Source Artifacts

- `.design-scratch/refactor-e2e/split-state-interface-impl-20260305-0847/shared-impact-analysis.md`
- `.design-scratch/refactor-e2e/split-state-interface-impl-20260305-0847/shared-module-structure.md`
- `.design-scratch/refactor-e2e/split-state-interface-impl-20260305-0847/shared-dependency-graph.md`
