# Task 2: Write E2E Consensus Integration Tests

**Status**: [x] complete
**Dependencies**: none
**Wave**: 1
**Complexity**: M

## Pre-Task Gate
N/A (no dependencies)

## Context
Add single-validator E2E tests that exercise the propose→verify→finalize path. They must fail under the stubbed engine because the vendor path and mailbox behavior are not yet wired.

## What to do
1. **Write failing test(s)**: Implement the tests in `crates/consensus-simplex/src/tests.rs`.
   - TC-007: `test_single_validator_produces_block` (height >= 1 within 30 seconds).
   - TC-008: `test_single_validator_with_transactions` (finalized block contains txs).
   - Run: `nix develop --command cargo test -p consensus-simplex -- test_single_validator_produces_block test_single_validator_with_transactions` and ensure failure.
2. **Implement**: N/A.
3. **Refactor**: N/A.
4. **Verify**: Append outputs to `.sisyphus/evidence/task-02-e2e-consensus-tests.txt`.

## Mock Boundary
**Allowed to mock**: Timeouts and harness logging.
**Must NOT mock**: `CommonwareEngine`, `Mailbox`, or `AppAdapter` reporter path.

## Must NOT do
- Do not modify `vendor/`.
- Do not change `crates/consensus/` traits.

## References
- `docs/design/real-simplex-consensus-wiring/TESTS.md`: TC-007, TC-008.
- `crates/consensus-simplex/src/engine.rs`: Implementation target.

## Acceptance Criteria
- `nix develop --command cargo test -p consensus-simplex -- test_single_validator_produces_block test_single_validator_with_transactions` fails before wiring.
- Evidence: `.sisyphus/evidence/task-02-e2e-consensus-tests.txt`

## Post-Task Gate
- Run: `nix develop --command cargo test -p consensus-simplex -- test_single_validator_produces_block test_single_validator_with_transactions`
- Expected: exit non-zero.
- Evidence MUST be appended to `.sisyphus/evidence/task-02-e2e-consensus-tests.txt`.

## QA Scenarios
1. Run the two E2E tests → they should fail in current stub.
   Evidence: `.sisyphus/evidence/task-02-e2e-consensus-tests.txt`

## Evidence
- `.sisyphus/evidence/task-02-e2e-consensus-tests.txt`
