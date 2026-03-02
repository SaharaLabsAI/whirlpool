# Task 01: state-commit-tests

## Summary
Write verification tests for state commitment and clone isolation in the `state` crate.

## Crate(s)
`state`

## Files Changed
`crates/state/src/db.rs`

## Dependencies
None

## Design Refs
`TESTS.md T-9, T-10, T-11, T-12`

## TDD Sequence
1. Write T-12: Verify `InMemoryStateDb::clone()` creates an isolated instance (Red)
2. Verify T-12 passes with existing code (Green - `Clone` is already present)
3. Write T-9: Verify `commit()` applies account changes from `BundleState` (Red)
4. Write T-10: Verify `commit()` applies storage changes (Red)
5. Write T-11: Verify `commit()` handles account destruction (Red)
6. Verify all tests pass with current implementation (Green)

## Implementation Details
Tests should be added to the `tests` module at the bottom of `crates/state/src/db.rs`. Use the existing `commit` method and check `accounts` and `bytecodes` maps directly to verify changes.

## Acceptance Criteria
- `nix develop --command cargo test -p state -- db::tests` passes
- `nix develop --command cargo build -p state` succeeds
- No new warnings

## Evidence
- Path: `.sisyphus/evidence/evm-tx-execution/01-state-commit-tests.log`
