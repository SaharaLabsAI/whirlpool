# Task 05: integration-test

## Summary
Write a full propose-verify round-trip integration test.

## Crate(s)
`app-evm`

## Files Changed
`crates/app-evm/tests/integration.rs`

## Dependencies
Task 04 (verify-execution)

## Design Refs
`TESTS.md T-13`

## TDD Sequence
1. Write T-13: Full genesis → propose → verify cycle with real everything (Red)
2. Verify T-13 passes with completed implementation (Green)

## Implementation Details
1. Create a mock `TxSource` with valid Ethereum transactions.
2. Initialize `InMemoryStateDb` with some starting balances.
3. Call `propose()` on the initial state to generate a block.
4. Call `verify()` with the generated block and initial state.
5. Verify `Ok(())` is returned and canonical state is correctly updated after `propose()`.

## Acceptance Criteria
- `nix develop --command cargo test -p app-evm -- integration` passes
- `nix develop --command cargo build -p app-evm` succeeds
- No new warnings

## Evidence
- Path: `.sisyphus/evidence/evm-tx-execution/05-integration-test.log`
