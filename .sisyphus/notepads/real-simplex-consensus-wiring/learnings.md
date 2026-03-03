## [2026-03-03T09:30] Task 1: Engine Unit Tests

### Test Structure
- Updated 4 unit tests to check observable engine behavior (status, height tracking)
- Tests correctly renamed to match test contracts TC-001 through TC-004
- All tests now assert `RunningEngine::status()` behavior instead of internal fields

### Failing Behavior (As Expected)
- `test_engine_can_be_constructed`: Panics with "Cannot drop a runtime in a context where blocking is not allowed"
- This is expected because the stub implementation uses tokio runtime incorrectly
- Evidence captured in `.sisyphus/evidence/task-01-engine-unit-tests.txt`

### Test Design Insights
- `RunningEngine::shutdown()` consumes `self`, so post-shutdown status cannot be checked on the same instance
- `status()` returns a **Copy** snapshot (not live view), so capturing status before shutdown and checking it after won't work
- The shutdown test correctly verifies: (1) engine is running before shutdown, (2) shutdown succeeds

### Next Steps
- Task 2 will add E2E integration tests
- Task 3 will replace the stub with real simplex::Engine wiring to make these tests pass

## [2026-03-03T09:45] Task 2: E2E Consensus Integration Tests

### Test Structure
- Extended TestBlock with transactions field and updated codec (CodecWrite/CodecRead/EncodeSize)
- Added BlockCollectorSink to capture finalized blocks for assertions
- Added MockTxApp that proposes blocks with transaction data
- Added TC-007: `test_single_validator_produces_block` — polls height for 30s
- Added TC-008: `test_single_validator_with_transactions` — polls finalized blocks for transactions

### Failing Behavior (As Expected)
- Both E2E tests hang/timeout waiting for finalization
- Tests run for over 60 seconds (exceeding timeout)
- Stub implementation doesn't drive real consensus, so height never advances beyond 0
- Evidence captured in `.sisyphus/evidence/task-02-e2e-consensus-tests.txt`

### Implementation Notes
- TestBlock transactions use length-prefixed encoding (count, then len+data per tx)
- BlockCollectorSink returns both the sink and the shared Vec for test access
- Tests poll with 500ms sleep intervals to observe state changes

### Next Steps
- Task 3 will replace the stub with real simplex::Engine wiring
- These E2E tests should pass once Task 3 is complete
