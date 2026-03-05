# ORACLE_PACK

## Invariants checklist
- Plan is bounded to split-state-interface-impl scope only (`state`, `state-memory`, `app-evm`, `whirlpool-node`).
- Task ordering must follow migration steps 1..6.
- Every task must include rollback + acceptance checks.
- Commands use `nix develop --command` wrappers for cargo operations.
- No source/design-doc edits are performed by plan generation.

## Destructive operations
- 1 op in 1 task (Task 02) - see `.scratch/DESTRUCTIVE_OPS.md`.

## Task snapshot
| Task | Dependency | Complexity |
|---|---|---|
| 01-lock-interface-surface-in-state | none | S |
| 02-scaffold-state-memory-crate | 01-lock-interface-surface-in-state | M |
| 03-move-concrete-db-and-revm-impls | 02-scaffold-state-memory-crate | L |
| 04-rewire-app-evm-to-state-memory | 03-move-concrete-db-and-revm-impls | L |
| 05-rewire-whirlpool-node-wrapper | 04-rewire-app-evm-to-state-memory | S |
| 06-remove-transitional-concrete-paths | 05-rewire-whirlpool-node-wrapper | M |

## TestID registry excerpt
TB-001..TB-006, TN-001..TN-006 are listed in INDEX Artifact Registry and task references.

## Design links
- `docs/refactor/split-state-interface-impl/INTENT.md`
- `docs/refactor/split-state-interface-impl/MIGRATION.md`
- `docs/refactor/split-state-interface-impl/TESTS.md`
- `docs/refactor/split-state-interface-impl/STRATEGY.md`

## Consumption rule
Read this ORACLE_PACK first. Then use targeted reads/grep on plan files only when a check fails.
