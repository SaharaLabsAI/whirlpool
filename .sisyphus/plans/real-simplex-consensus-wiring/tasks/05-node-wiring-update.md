# Task 5: Update whirlpool-node Wiring

**Status**: [ ] pending
**Dependencies**: Task 3
**Wave**: 3
**Complexity**: S

## Pre-Task Gate
- Run: `nix develop --command cargo test -p consensus-simplex -- test_engine_can_be_constructed test_engine_starts_with_real_simplex test_engine_shutdown_aborts_handle test_engine_status_tracks_height`
- Expected: exit 0
- If any gate command fails: **STOP. Do NOT start this task.**

## Context
Align `crates/whirlpool-node/src/main.rs` with the real `CommonwareEngine` constructor, pass signer/validators, and keep the `OracleHandle` alive for the blocker.

## What to do
1. **Write failing test(s)**: N/A (binary wiring only).
2. **Implement**: Update `main.rs`:
   - Pass `CommonwareConfig` signer/validators to `CommonwareEngine::new`.
   - Keep `OracleHandle` around and use `control(public_key)` to construct the blocker passed to the engine.
   - Update runtime wiring to reflect real-start semantics.
3. **Refactor**: Remove any stub wiring or unused modules.
4. **Verify**: Run `nix develop --command cargo build --workspace` and `nix develop --command cargo test --workspace`, appending output to `.sisyphus/evidence/task-05-node-wiring-update.txt`.

## Mock Boundary
**Allowed to mock**: None.
**Must NOT mock**: Runtime wiring parts used in `main.rs`.

## Must NOT do
- Do not edit `vendor/`.
- Do not change `crates/consensus/` traits.

## References
- `docs/design/real-simplex-consensus-wiring/FLOWS.md`: Flow 1/3.
- `crates/whirlpool-node/src/main.rs`: Implementation target.

## Acceptance Criteria
- `nix develop --command cargo build --workspace` passes.
- `nix develop --command cargo test --workspace` passes.
- Evidence: `.sisyphus/evidence/task-05-node-wiring-update.txt`

## Post-Task Gate
- Run: `nix develop --command cargo build --workspace && nix develop --command cargo test --workspace`
- Expected: exit 0
- Evidence MUST be appended to `.sisyphus/evidence/task-05-node-wiring-update.txt`.

## QA Scenarios
1. Run workspace tests after node wiring update → entire suite must pass, proving integration.
   Evidence: `.sisyphus/evidence/task-05-node-wiring-update.txt`

## Evidence
- `.sisyphus/evidence/task-05-node-wiring-update.txt`
