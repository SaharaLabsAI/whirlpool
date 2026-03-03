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
