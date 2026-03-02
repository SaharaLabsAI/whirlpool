# Task 04: verify-execution

## Summary
Replace the `verify()` stub in `app-evm` with full transaction re-execution and state root validation.

## Crate(s)
`app-evm`

## Files Changed
`crates/app-evm/src/executor.rs`

## Dependencies
Task 03 (propose-execution)

## Design Refs
`FLOWS.md §F2`, `TESTS.md T-4, T-5, T-6, T-8`

## TDD Sequence
1. Write T-4: `verify_accepts_valid_block` (Red)
2. Implement verify re-execution flow — decode, clone, BasicBlockExecutor, 4-field compare (Green for T-4)
3. Write T-5: `verify_rejects_wrong_state_root` (Red)
4. Verify T-5 passes with existing implementation (Green for T-5, no new production code expected)
5. Write T-6: `verify_rejects_undecodable_transactions` (Red)
6. If T-6 fails, add decode-all-or-fail guard at start of verify (Green for T-6)
7. Write T-8: `verify_rejects_wrong_gas_used` (Red)
8. If T-8 fails, ensure gas_used comparison is included in 4-field check (Green for T-8)

## Implementation Details
1. Reconstruct a `RecoveredBlock` from the proposed block.
2. Clone `state_db` for isolation (never committed to canonical state).
3. Use `BasicBlockExecutor::execute_one(&recovered_block)` to re-execute.
4. Obtain `BlockExecutionResult` containing `BundleState`, `receipts`, `gas_used`.
5. Apply the `BundleState` to the cloned state instance.
6. Compute all 4 fields: `state_root`, `tx_root`, `receipts_root`, `gas_used`.
7. Fail if any computed field does not match the proposed block.

## Acceptance Criteria
- `nix develop --command cargo test -p app-evm -- verify` passes
- `nix develop --command cargo build -p app-evm` succeeds
- No new warnings

## Evidence
- Path: `.sisyphus/evidence/evm-tx-execution/04-verify-execution.log`
