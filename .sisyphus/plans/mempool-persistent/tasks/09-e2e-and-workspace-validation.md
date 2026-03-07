# Task 09: e2e-and-workspace-validation

**Status**: pending
**Dependencies**: 08
**Wave / Phase**: Wave 7 / Phase 7 (end-to-end validation)
**Complexity**: S
**Target Crate(s)**: workspace-wide
**AC IDs**: AC-1, AC-2

## Objective
Run final end-to-end and full-workspace validation to confirm implementation correctness and regression safety.

## Design Refs
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/SUMMARY.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/TESTS.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/proven-ac.md`

## Steps
1. Run end-to-end persistence tests (restart survival + proposal inclusion).
2. Run full workspace build gate.
3. Run full workspace test gate.
4. Confirm no vendor modifications and all AC IDs are covered by plan execution.

## Atomic Verification
- `nix develop --command cargo build`
- `nix develop --command cargo test`

## Done When
- Workspace build and tests pass.
- End-to-end persistence behavior is validated and evidence recorded.
