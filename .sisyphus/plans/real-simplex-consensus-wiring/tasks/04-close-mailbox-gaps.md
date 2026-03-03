# Task 4: Close Mailbox/MailboxActor Gaps

**Status**: [ ] pending
**Dependencies**: Task 3
**Wave**: 3
**Complexity**: M

## Pre-Task Gate
- Run: `nix develop --command cargo test -p consensus-simplex -- test_engine_can_be_constructed test_engine_starts_with_real_simplex test_engine_shutdown_aborts_handle test_engine_status_tracks_height`
- Expected: exit 0
- If any gate command fails: **STOP. Do NOT start this task.**

## Context
Tighten `Mailbox`/`MailboxActor` so they drive `ConsensusApp::{genesis, propose, verify}` with no heuristics, enabling real block production and finalized height progression.

## What to do
1. **Write failing test(s)**: Re-run TC-007/TC-008 to capture failure before implementation (evidence must be accumulated in `.sisyphus/evidence/task-04-close-mailbox-gaps.txt`).
2. **Implement**: Update `crates/consensus-simplex/src/mailbox.rs`:
   - Ensure `MailboxActor::run` forwards propose/verify/finalize requests directly to `ConsensusApp`.
   - Implement `Relay::broadcast` to actually dispatch `ConsensusMessage`s to the vendor channels.
   - Wire `FinalizationSink` acknowledgements so height is surfaced in `RunningEngine::status()`.
3. **Refactor**: Remove digest-heuristic shortcuts and ensure the actor handles multi-step flows.
4. **Verify**: Run `nix develop --command cargo build --workspace && nix develop --command cargo test --workspace` and append the passing run to `.sisyphus/evidence/task-04-close-mailbox-gaps.txt`.

## Mock Boundary
**Allowed to mock**: `ConsensusApp` behaviors in mailbox unit tests.
**Must NOT mock**: The mailbox channel interface consumed by `CommonwareEngine`.

## Must NOT do
- Do not edit `vendor/`.
- Do not change `crates/consensus/` traits.

## References
- `docs/design/real-simplex-consensus-wiring/FLOWS.md`: Flow 2 details Message flow.
- `crates/consensus-simplex/src/mailbox.rs`: Implementation target.

## Acceptance Criteria
- `nix develop --command cargo test -p consensus-simplex -- test_single_validator_produces_block test_single_validator_with_transactions` passes.
- `nix develop --command cargo build --workspace && nix develop --command cargo test --workspace` passes.
- Evidence: `.sisyphus/evidence/task-04-close-mailbox-gaps.txt`

## Post-Task Gate
- Run: `nix develop --command cargo build --workspace && nix develop --command cargo test --workspace`
- Expected: exit 0
- Evidence MUST be appended to `.sisyphus/evidence/task-04-close-mailbox-gaps.txt`.

## QA Scenarios
1. Run the TC-007/TC-008 suite after mailbox fixes → should pass within the 30s windows.
   Evidence: `.sisyphus/evidence/task-04-close-mailbox-gaps.txt`

## Evidence
- `.sisyphus/evidence/task-04-close-mailbox-gaps.txt`
