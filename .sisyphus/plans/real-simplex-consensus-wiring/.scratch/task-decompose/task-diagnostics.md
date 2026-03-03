# Task Stuck Diagnosis: Task 03 — Replace Engine Start Stub with Real Simplex Wiring

## Metadata
- **Plan**: real-simplex-consensus-wiring
- **Task file**: tasks/03-real-simplex-wiring.md
- **Date**: 2026-03-03
- **Status**: STUCK

## Failure Class: `complexity_underestimate` + `stale_grounding`

## Evidence Summary

### 1. Stale Test Name References (stale_grounding)
Task 3's Pre-Task Gate references 3 tests that **do not exist** in the codebase:
- `test_engine_starts_with_real_simplex` → **MISSING**
- `test_engine_shutdown_aborts_handle` → **MISSING**
- `test_engine_status_tracks_height` → **MISSING**

Task 1 was marked complete but created **differently-named** tests:
- `test_engine_start_and_status` (in tests.rs)
- `test_engine_shutdown` (in tests.rs)
- `test_engine_height_tracking` (in tests.rs)

The only matching name (`test_engine_can_be_constructed`) exists in `engine.rs` and currently **FAILS**.

### 2. Engine Constructor Broken
`test_engine_can_be_constructed` fails with:
> "Cannot drop a runtime in a context where blocking is not allowed. This happens when a runtime is dropped from within an asynchronous context."

The `test_context()` helper creates a `commonware_tokio::Context` via `spawn_blocking` + `Runner::default().start()`. The resulting context holds a runtime that panics when dropped inside a tokio async test.

### 3. Engine `start()` Is Entirely Stubbed
Current `start()` implementation (engine.rs lines 88-164):
- Calls `network.start()` (single sender/receiver) instead of `network.start_per_channel()` (3 pairs: vote/cert/resolver)
- Creates Mailbox, MailboxActor, FinalizationSink but prefixes all with `_` (unused)
- Spawns a `std::thread` simulating finalization every 5 seconds by incrementing an `AtomicU64`
- **Never** calls `simplex::Engine::new(...)` or `.start(...)`

The real implementation requires:
1. `network.start_per_channel()` → 3 sender/receiver pairs
2. `AppAdapter::new(app, sink)` construction
3. `simplex::Config` building (signer, elector, blocker, automaton=Mailbox, relay=Mailbox, reporter=AppAdapter, strategy, timing)
4. `simplex::Engine::new(context, config).start(vote, cert, resolver)` → Handle
5. Remove the simulation thread entirely

### 4. Scope Exceeds M Complexity
The task was estimated as M (2-4 files). Actual scope:
| File | Change Required |
|------|----------------|
| `engine.rs` | Major rewrite of `start()`, fix constructor runtime issue, remove stub thread |
| `tests.rs` | Fix test names to match plan, fix async runtime setup, update test expectations |
| `mailbox.rs` | `Relay::broadcast` is a no-op — needs real broadcast for multi-node |
| `adapter.rs` | Wire AppAdapter into engine start (currently unused) |
| `config.rs` | Potentially extend for simplex::Config building |
| `p2p provider.rs` | Consumer of `start_per_channel()` — integration point |

**6 files touched** = exceeds M complexity.

### 5. Hanging Tests
6 tests hang for 60+ seconds in evidence logs:
- `test_engine_can_start_and_shutdown`
- `test_engine_simulates_block_finalization`
- `test_engine_height_tracking`
- `test_engine_opens_three_channels`
- `test_engine_start_and_status`
- `test_engine_shutdown`

These hang because the stub thread simulates finalization at 5s intervals, but tests have misaligned timing expectations or the tokio runtime/thread interaction causes deadlocks.

### 6. E2E Tests Also Hang
From Task 2 evidence:
- `test_single_validator_produces_block` — hung 60+ seconds
- `test_single_validator_with_transactions` — hung 60+ seconds

These use MockNetworkProvider which won't support real simplex message passing.

## Affected Files
- `crates/consensus-simplex/src/engine.rs` (primary — major rewrite)
- `crates/consensus-simplex/src/tests.rs` (test name mismatch, hanging tests)
- `crates/consensus-simplex/src/mailbox.rs` (Relay::broadcast no-op)
- `crates/consensus-simplex/src/adapter.rs` (AppAdapter wiring)
- `crates/consensus-simplex/src/sink.rs` (FinalizationSink wiring)
- `crates/p2p-commonware/src/provider.rs` (start_per_channel consumer)

## Recommended Action: **DECOMPOSE**

The task combines 4+ distinct concerns:
1. Fix the constructor/runtime issue so engine can be constructed in tests
2. Wire per-channel networking into engine start (replace single-channel stub)
3. Build simplex::Config and start real vendor engine (replace simulation thread)
4. Fix/align test names and expectations with what Tasks 1/2 actually created

Each is independently testable and should be a separate S or M sub-task.
