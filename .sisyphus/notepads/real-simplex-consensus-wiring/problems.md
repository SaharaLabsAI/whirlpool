## [2026-03-03 RESOLVED] Task 03.3: Type Wrapper Mismatch Prevents Vendor Integration

**Status**: ✅ RESOLVED

**Problem**: `PerChannelNetwork` returned wrapped types that did NOT implement vendor traits required by `simplex::Engine::start()`.

**Resolution**: Removed wrappers from `PerChannelNetwork` to expose raw vendor types. See `.sisyphus/notepads/real-simplex-consensus-wiring/decisions.md` for full analysis.

**Impact**: Eliminated 6 compilation errors, unblocked Task 03.3 implementation.

---

## [2026-03-03 BLOCKER] Task 03.3: AppAdapter Reporter Type Mismatch (Task 4 Scope)

**Status**: ⏸️ BLOCKED - Belongs to Task 4 scope

**Problem**: `AppAdapter` implements `Reporter::Activity = Update<B>` (our vendor-agnostic type) but vendor `simplex::Config` expects `Reporter::Activity = Activity<Scheme, Digest>` (vendor-specific type).

**Error**:
```
error[E0271]: type mismatch resolving `<AppAdapter<..., ..., ..., _> as Reporter>::Activity == Activity<..., ...>`
note: expected this to be `Activity<Scheme, Digest>`
      found `Update<<A as ConsensusApp>::Block>`
```

**Root Cause**: `AppAdapter` was designed to bridge our vendor-agnostic `ConsensusApp` to vendor Reporter trait, but the Activity associated type doesn't match. The adapter needs to convert between our `Update<B>` events and vendor `Activity<Scheme, Digest>` events.

**Location**: `crates/consensus-simplex/src/adapter.rs` line 125

**Impact**: Prevents final compilation of Task 03.3. Engine wiring is complete, but cannot be instantiated until Reporter trait is properly implemented.

**Resolution Path**: Task 4 (Close Mailbox/MailboxActor Gaps) will fix the AppAdapter Reporter implementation. Task spec line 18-21:
> "Update `crates/consensus-simplex/src/mailbox.rs`:
> - Ensure `MailboxActor::run` forwards propose/verify/finalize requests directly to `ConsensusApp`.
> - Implement `Relay::broadcast` to actually dispatch `ConsensusMessage`s to the vendor channels.
> - Wire `FinalizationSink` acknowledgements so height is surfaced in `RunningEngine::status()`."

AppAdapter is tightly coupled to Mailbox/MailboxActor implementation, so fixing them together in Task 4 makes architectural sense.

**Workaround**: None. This is a fundamental type system constraint. Task 03.3 cannot compile until AppAdapter implements the vendor Reporter trait correctly.

**Next Steps**:
1. Task 03.4 can proceed independently (test renaming)
2. Task 4 must fix AppAdapter + Mailbox together
3. Task 03.3 will be complete once Task 4 unblocks compilation

**Files Affected**:
- `crates/consensus-simplex/src/adapter.rs` (Reporter impl)
- `crates/consensus-simplex/src/mailbox.rs` (Automaton, Relay impls)
- `crates/consensus-simplex/src/engine.rs` (consumer - ready once blocker fixed)

## [2026-03-03 UNRESOLVED] Task 03.4: Integration Tests Timeout

**Status**: ⚠️ KNOWN ISSUE - Out of scope for Task 03.4

**Problem**: After wiring real vendor `simplex::Engine`, integration tests hang and timeout after 120 seconds.

**Symptoms**:
- Tests compile successfully ✅
- Tests start without panics ✅
- Tests hang during execution (no progress, no output) ❌
- Test runner kills tests after 120s timeout

**Affected Tests**:
- `test_engine_starts_with_real_simplex`
- `test_engine_shutdown_aborts_handle`
- `test_engine_status_tracks_height`

**Hypothesis**:
Real vendor `simplex::Engine` may require:
1. Network message simulation (proposals, notarizations, etc.)
2. Quorum of validators (tests use single validator)
3. Explicit shutdown signaling
4. Different timeout thresholds for consensus progression

**Impact**:
- Does NOT block Task 03.4 completion (renaming + compilation done)
- May indicate integration issues for Task 5 (whirlpool-node wiring)
- Should be investigated before production deployment

**Resolution Path**:
1. Review vendor `simplex::Engine` documentation for test patterns
2. Check vendor test examples for single-validator scenarios
3. Add test-specific engine configuration (shorter timeouts, mock messages)
4. Consider separate unit tests (fast, focused) vs E2E tests (slow, real engine)

**Workaround**: Task 03.4 acceptance criteria focus on compilation and test names, both achieved. Timeout debugging deferred to Task 5 integration or explicit user request.

**Files**: `crates/consensus-simplex/src/tests.rs` (lines 347-400+)
