# Diagnosis: Task 3 — Replace Engine Start Stub with Real Simplex Wiring

## Failure Class: `complexity_underestimate` + `stale_grounding`

## Evidence Summary

1. **Test name mismatch (stale grounding)**: Task 3's Pre-Task Gate references 3 tests that **do not exist**:
   - `test_engine_starts_with_real_simplex` → MISSING
   - `test_engine_shutdown_aborts_handle` → MISSING
   - `test_engine_status_tracks_height` → MISSING
   
   Task 1 was completed but created **different** test names:
   - `test_engine_start_and_status`
   - `test_engine_shutdown`
   - `test_engine_height_tracking`
   
   The only matching name (`test_engine_can_be_constructed`) exists in `engine.rs` and currently **FAILS** with a tokio runtime panic.

2. **Engine constructor broken**: `test_engine_can_be_constructed` fails with: *"Cannot drop a runtime in a context where blocking is not allowed"* — the engine constructor or its drop path creates/destroys a tokio runtime incorrectly.

3. **Engine start() is entirely stubbed**: The current `start()` impl:
   - Calls `network.start()` (single sender/receiver) instead of `network.start_per_channel()` (3 pairs)
   - Creates Mailbox/MailboxActor/FinalizationSink but prefixes all with `_` (unused)
   - Spawns a `std::thread` simulating finalization every 5s by incrementing an AtomicU64
   - Never calls `simplex::Engine::new(...).start(...)`
   
   Replacing this requires: per-channel network wiring, AppAdapter construction, simplex::Config building, vendor engine startup, and removing the simulation thread.

4. **Scope is larger than M complexity**: The task touches:
   - `engine.rs` (major rewrite of `start()`, fix constructor, remove stub thread)
   - `mailbox.rs` (Relay::broadcast is no-op, actor has simplifications)
   - `tests.rs` (fix test names, fix async runtime issues)
   - `adapter.rs` (wire AppAdapter into engine start)
   - `config.rs` (potentially extend for simplex::Config building)
   - Multiple cross-cutting concerns: async runtime management, per-channel networking, vendor API integration

5. **Hanging tests**: 6 tests hang for 60+ seconds because the stub thread simulates finalization but the test assertions or vendor expectations are misaligned.

## Affected Files

- `crates/consensus-simplex/src/engine.rs` (primary)
- `crates/consensus-simplex/src/tests.rs` (test name mismatch, hanging tests)
- `crates/consensus-simplex/src/mailbox.rs` (Relay::broadcast no-op)
- `crates/consensus-simplex/src/adapter.rs` (AppAdapter wiring)
- `crates/consensus-simplex/src/sink.rs` (FinalizationSink wiring)
- `crates/p2p-commonware/src/provider.rs` (start_per_channel consumer)

## Recommended Action: `decompose`

The task combines 4+ distinct concerns into one M-sized task:
1. Fix the constructor/runtime issue
2. Wire per-channel networking into engine start
3. Build simplex::Config and start vendor engine
4. Fix/align test names with what Tasks 1/2 actually created

Each of these is independently testable and should be a separate sub-task.
