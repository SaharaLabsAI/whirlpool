# INDEX — Split State Interface/Implementation

## Reading Guide

1. Start with `SUMMARY.md` for final verdict and decision rationale.
2. Read `INTENT.md` in scratch for objective, symbol scope, and success criteria.
3. Read `IMPACT.md` for blast radius and risk seams.
4. Read `STRATEGY.md` for sequencing constraints and rollback posture.
5. Read `MIGRATION.md` for Step 1-6 execution order and verification gates.
6. Read `TESTS.md` for migration-to-test contract mapping.
7. Read per-crate `CHANGES.md` files for file-level edits by crate.
8. Check `BLOCKERS.md` for active or resolved blockers.

## File Inventory

### Tier 1 (Core narrative and decisions)
- `BLOCKERS.md` (14 lines) — final blocker ledger; no active blockers.
- `IMPACT.md` (69 lines) — blast radius, seam risk, dependency impact, unknowns.
- `MIGRATION.md` (105 lines) — ordered Step 1-6 migration with verification and rollback.
- `STRATEGY.md` (55 lines) — interface-first split strategy, risk mitigation, rollback strategy.
- `SUMMARY.md` (40 lines) — final synthesis and PASS/REVISE rationale.
- `TESTS.md` (85 lines) — `TB-*` breakage map, `TN-*` contracts, verification sequence.

### Tier 2 (Per-crate implementation slices)
- `app-evm/CHANGES.md` (34 lines) — concrete import rewiring to `state-memory`.
- `state-memory/CHANGES.md` (33 lines) — new implementation crate ownership and exports.
- `state/CHANGES.md` (35 lines) — interface-only state crate surface.
- `whirlpool-node/CHANGES.md` (32 lines) — node wrapper rewiring to `state-memory`.

### Tier 3 (Scratch and process state)
- `.design-scratch/.../INTENT.md` (51 lines) — source-of-truth intent and symbol table.
- `.design-scratch/.../run-state.md` (updated in finalize) — step/status/verdict state machine.
- `.design-scratch/.../final-self-check.md` (this run) — consistency self-check report.
- `.design-scratch/.../finalization-notes.md` (this run) — sub-refactor continuation decision.

## Notes

- In-scope crates: `state`, `state-memory`, `app-evm`, `whirlpool-node`.
- Migration step count: 6 (`MIGRATION.md`).
- Test contract count: 6 (`TN-001` through `TN-006`, `TESTS.md`).
