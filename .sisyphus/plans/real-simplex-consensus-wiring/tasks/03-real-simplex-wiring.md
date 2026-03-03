# Task 3: Replace Engine Start Stub with Real Simplex Wiring

**Status**: [ ] pending
**Dependencies**: Task 1, Task 2
**Wave**: 2
**Complexity**: M

## Pre-Task Gate
- Run: `nix develop --command cargo test -p consensus-simplex -- test_engine_can_be_constructed test_engine_starts_with_real_simplex test_engine_shutdown_aborts_handle test_engine_status_tracks_height`
- Expected: exit non-zero
- If any gate command fails: **STOP. Do NOT start this task.**

## Context
Replace the stubbed `CommonwareEngine::start()` with the real Simplex wiring, sourcing per-channel networking from `CommonwareNetworkProvider` and binding the Mailbox to vendor `Config`.

## What to do
1. **Write failing test(s)**: Already covered by Tasks 1 and 2.
2. **Implement**: Update `crates/consensus-simplex/src/engine.rs`:
   - Call `network.start_per_channel()` for vote/cert/resolver.
   - Create mailbox channel and spawn `MailboxActor`.
   - Build `simplex::Config` with the signer/validators, blocker from `OracleHandle.control`, and adapters (`Mailbox`, `AppAdapter`, `FinalizationSink`).
   - Start vendor engine: `simplex::Engine::new(context, config).start(vote, cert, resolver)`.
   - Return `RunningEngine` backed by the vendor handle and shared height sink.
3. **Refactor**: Remove the stub thread that simulated finalization.
4. **Verify**: Run `nix develop --command cargo build --workspace && nix develop --command cargo test --workspace` and append the passing run to `.sisyphus/evidence/task-03-real-simplex-wiring.txt`.

## Mock Boundary
**Allowed to mock**: `ConsensusApp` and provider helpers inside unit tests.
**Must NOT mock**: `commonware_consensus::simplex::Engine` startup (`Engine::new(...).start(...)`).

## Must NOT do
- Do not edit `vendor/`.
- Do not touch `crates/consensus/` trait definitions.

## References
- `docs/design/real-simplex-consensus-wiring/FLOWS.md`: Flow 1 outlines the wiring steps.
- `crates/p2p-commonware/src/provider.rs`: `start_per_channel()`.
- `crates/consensus-simplex/src/mailbox.rs`: Mailbox/actor integration.

## Acceptance Criteria
- `nix develop --command cargo test -p consensus-simplex -- test_engine_can_be_constructed test_engine_starts_with_real_simplex test_engine_shutdown_aborts_handle test_engine_status_tracks_height` passes.
- `nix develop --command cargo build --workspace && nix develop --command cargo test --workspace` passes.
- Evidence: `.sisyphus/evidence/task-03-real-simplex-wiring.txt`

## Post-Task Gate
- Run: `nix develop --command cargo build --workspace && nix develop --command cargo test --workspace`
- Expected: exit 0
- Evidence MUST be appended to `.sisyphus/evidence/task-03-real-simplex-wiring.txt`.

## QA Scenarios
1. Run Task 1 unit tests post-wiring → they should pass now.
   Evidence: `.sisyphus/evidence/task-03-real-simplex-wiring.txt`

## Evidence
- `.sisyphus/evidence/task-03-real-simplex-wiring.txt`
