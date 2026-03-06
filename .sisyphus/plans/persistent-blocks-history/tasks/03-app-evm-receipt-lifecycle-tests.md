# Task 03: app-evm-receipt-lifecycle-tests

**Status**: pending
**Dependencies**: 01, 02
**Wave**: 2
**Complexity**: M
**Target Crate(s)**: app-evm (role: test)

## Pre-Task Gate
- `nix develop --command cargo build -p app` succeeds.
- `nix develop --command cargo build -p state` succeeds.

## Context
Receipts are generated during block proposal but must be persisted during finalization. This task creates the test suite that verifies `app-evm` correctly captures these receipts in `propose()` and handles them in `store_finalized_block()`. These tests are written before the actual implementation (TDD).

## What to do

### TDD Flow
1. Write failing tests for `propose()` to verify it returns non-None receipts.
2. Write failing tests for `store_finalized_block()` to verify it calls `BlockStorage::store_block` with the correct receipts.
3. Write failing tests for edge cases like missing receipts and persistence errors.
4. Verify tests fail to compile (expected due to missing fields/methods).

### Specific steps
1. Edit `crates/app-evm/src/executor.rs` and add tests for:
   - `TC-AE-01`: `propose()` returns non-None receipts with a count matching the transaction count.
   - `TC-AE-02`: `store_finalized_block()` correctly calls `store_block` on a mock `BlockStorage` and clears the pending receipts.
   - `TC-AE-03`: `store_finalized_block()` handles the case where no receipts are pending (e.g., node just started and finalized an existing block).
   - `TC-AE-04`: `store_finalized_block()` handles a `BlockStorage` error by logging and returning a `State` error variant.

## Mock Boundary
- Use a mock implementation of the `BlockStorage` trait to verify interaction.

## Must NOT do
- Do NOT implement the actual capture or persistence logic in this task.
- Do NOT change existing test cases in `executor.rs`.

## References
- `.design-scratch/e2e/persistent-blocks-history-20260306-1500/docs/TESTS.md`
- `docs/crates/app-evm.md`

## Acceptance Criteria
- `nix develop --command cargo test -p app-evm` (tests should fail to compile or run, proving the need for Task 04).

## Post-Task Gate
- Command: `nix develop --command cargo test -p app-evm`
- Expected: exit 1 (proving tests fail as expected)
- Max retries: 1

## Post-Task Reconciliation
- Update Artifact Registry in INDEX.md (TC-AE-01..04 status: pending_impl).

## QA Scenarios
- QA-5: Receipts captured from propose.

## Evidence
`.sisyphus/evidence/task-03-app-evm-receipt-lifecycle-tests.txt`
