# INDEX — Split Interface From Implementation

## Reading Guide

1. Start with `SUMMARY.md` for an executive go/no-go view.
2. Read `INTENT.md` for scope, symbol matrix, and success criteria.
3. Read `IMPACT.md` for blast radius and cross-crate risks.
4. Read `STRATEGY.md` for wave ordering, constraints, and rollback posture.
5. Read `MIGRATION.md` for Step 1-9 execution details.
6. Read `TESTS.md` for migration-aligned coverage and verification commands.
7. Use per-crate `CHANGES.md` files for crate-local implementation slices.
8. Check `BLOCKERS.md` before execution handoff.

## File Inventory

### Tier 1 (Core narrative and decisions)
- `SUMMARY.md` (11 lines) — executive synthesis and readiness recommendation.
- `INTENT.md` (168 lines) — objective, scope, symbols, constraints, and success criteria.
- `IMPACT.md` (68 lines) — blast radius, seam risk, dependency impact, and unknowns.
- `STRATEGY.md` (68 lines) — interface-first plan, wave ordering, risk mitigation, rollback.

### Tier 2 (Execution plan and crate slices)
- `MIGRATION.md` (150 lines) — incremental 9-step migration with verification/rollback.
- `TESTS.md` (94 lines) — `TB-*` breakage map, `TN-*` additive contracts, verification spine.
- `app/CHANGES.md` (31 lines) — tx-source extraction from `app::traits`.
- `consensus/CHANGES.md` (35 lines) — trait consolidation under `consensus::traits`.
- `p2p/CHANGES.md` (30 lines) — stabilization of interface-only `p2p::traits`.
- `state/CHANGES.md` (31 lines) — additive `state::traits::StateDb` introduction.
- `consensus-simplex/CHANGES.md` (32 lines) — `CommonwareBlock` relocation.
- `p2p-commonware/CHANGES.md` (31 lines) — additive `CommonwareTransport` interface.
- `app-evm/CHANGES.md` (32 lines) — `StateProvider` relocation to `app-evm::traits`.

### Tier 3 (Gate/status artifacts)
- `BLOCKERS.md` — blocker register (active + resolved).
- `INDEX.md` — this file inventory and loading guide.

## Notes

- Tiering follows `rust-whiteboard-refactor/shared/conventions.md`.
- Scope includes crates: `app`, `consensus`, `p2p`, `state`, `consensus-simplex`, `p2p-commonware`, `app-evm`.
