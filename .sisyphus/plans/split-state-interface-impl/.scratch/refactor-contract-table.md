# Refactor Contract Table

## Objective
Split the current `state` crate into two physical crates so interface-only consumers can depend on `state`, while concrete in-memory DB users depend on `state-memory`.

## Scope
| Crate | Role | Change Type | Risk Level |
|-------|------|------------|------------|
| state | source/target | restructure + cleanup exports | medium |
| state-memory | target | create + move concrete implementation | high |
| app-evm | consumer | import/dependency rewire | high |
| whirlpool-node | consumer | import/dependency rewire | medium |

## Migration Steps -> Task Mapping
| Step # | Description | Affected Crates | Estimated Complexity | Test Contracts |
|--------|------------|-----------------|---------------------|----------------|
| 1 | Lock Interface Surface in `state` | state | S | TB-001, TN-001, TN-002 |
| 2 | Scaffold `state-memory` Crate | state-memory, workspace | M | TB-002 |
| 3 | Move Concrete DB + `revm` Impl Blocks | state, state-memory | L | TB-003, TN-003 |
| 4 | Rewire `app-evm` to Concrete Crate | app-evm, state-memory, state | M | TB-004, TN-004 |
| 5 | Rewire `whirlpool-node` Runtime Wrapper | whirlpool-node, state-memory, state | S | TB-005, TN-005 |
| 6 | Remove Transitional Concrete Paths from `state` | state, app-evm, whirlpool-node, state-memory | M | TB-006, TN-006 |

## Cross-cutting Concerns
- Preserve one-way layering (`state-memory -> state`) and prevent reverse edges during Cargo/dependency updates.
- Keep `StateDb`/`StateError` contracts stable while moving only concrete implementation symbols.
- Maintain behavior parity for in-memory DB logic (`state_root`, `commit`, storage and revm integration) through move-only semantics.
- Execute rewiring in migration order so each step remains compilation-safe.

## Rollback Dependencies
- Step 2 rollback assumes Step 1 remains intact.
- Step 3 rollback depends on Step 2 crate scaffolding state.
- Step 4 rollback depends on Step 3 concrete exports existing in `state-memory`.
- Step 5 rollback depends on Step 4 `app-evm` transition state and Step 3 exports.
- Step 6 rollback may require bounded compatibility re-exports and should respect reverse-order rollback from steps 5 -> 1.
