# Task 1: Write Engine Unit Tests

**Status**: [ ] pending
**Dependencies**: none
**Wave**: 1
**Complexity**: S

## Pre-Task Gate
N/A (no dependencies)

## Context
Align unit tests with observable behavior of the engine lifecycle (status reporting, finalized height updates, and clean shutdown) instead of poking at private internals. These tests encode the intent that a real Simplex handle must exist for the engine to satisfy its contracts.

## What to do
1. **Write failing test(s)**: Update each test to assert the observable state transitions rather than internal handle fields.
   - File: `crates/consensus-simplex/src/tests.rs`
   - `test_engine_can_be_constructed`: Assert `RunningEngine::status()` returns `Running` and the shared `FinalizationSink` height remains at zero until real finalization occurs.
   - `test_engine_starts_with_real_simplex`: Assert the engine reports `Running` and height updates depend on the sink instead of the stub thread.
   - `test_engine_shutdown_aborts_handle`: Assert `RunningEngine::shutdown()` changes the status to `Stopped` and that `status()` no longer reports `Running`.
   - Run: `nix develop --command cargo test -p consensus-simplex -- test_engine_can_be_constructed test_engine_starts_with_real_simplex test_engine_shutdown_aborts_handle test_engine_status_tracks_height` to confirm these assertions fail against the stub.
2. **Implement**: N/A — this task only grounds the tests in failure before wiring.
3. **Refactor**: N/A.
4. **Verify**: Capture the failing runs to `.sisyphus/evidence/task-01-engine-unit-tests.txt` before moving on.

## Mock Boundary
**Allowed to mock**: `ConsensusApp`, `EventSink`, and basic network provider behavior within these unit tests.
**Must NOT mock**: `CommonwareEngine::start()` should remain real so the tests fail on the stub path.

## Must NOT do
- Do not modify files under `vendor/`.
- Do not change consensus traits in `crates/consensus/`.

## References
- `docs/design/real-simplex-consensus-wiring/FLOWS.md`: Engine Startup.
- `crates/consensus-simplex/src/engine.rs`: Implementation target.

## Acceptance Criteria
- `nix develop --command cargo test -p consensus-simplex -- test_engine_can_be_constructed test_engine_starts_with_real_simplex test_engine_shutdown_aborts_handle test_engine_status_tracks_height` fails reproducibly.
- Evidence: `.sisyphus/evidence/task-01-engine-unit-tests.txt`

## Post-Task Gate
- Run: `nix develop --command cargo test -p consensus-simplex -- test_engine_can_be_constructed test_engine_starts_with_real_simplex test_engine_shutdown_aborts_handle test_engine_status_tracks_height`
- Expected: exit non-zero (stub still in place).
- Evidence MUST be appended to `.sisyphus/evidence/task-01-engine-unit-tests.txt`.

## QA Scenarios
1. Run the targeted unit tests → all should fail on the stub implementation.
   Evidence: `.sisyphus/evidence/task-01-engine-unit-tests.txt`

## Evidence
- `.sisyphus/evidence/task-01-engine-unit-tests.txt`
