# Real Simplex Consensus Wiring

## TL;DR
**Summary**: Replace the `CommonwareEngine` stub with real vendor `simplex::Engine` wiring, close Mailbox/Actor gaps, and sync the node binary so a single-validator run produces finalized blocks.
**Deliverables**:
- Ground the existing unit tests (TC-001 through TC-004) on observable consensus handles.
- Add failing single-validator E2E tests (TC-007/TC-008) that require real propose→verify→finalize cycles.
- Wire `CommonwareEngine::start()` to real vendor per-channel networking, the Mailbox Actor path, and the Oracle blocker.
- Tighten Mailbox/MailboxActor behavior and update `whirlpool-node` to the new engine shape.
**Effort**: 8 tasks (3×S, 5×M) — original Task 3 decomposed into 4 sub-tasks (03.1–03.4).
**Parallel**: None — tasks modify the same `tests.rs` harness and workflow, and Waves 3/4/5 need workspace-wide verification.
**Critical Path**: T1 → T2 → T03.1 → T03.2 → T03.3 → T03.4 → T4 → T5.

## Context
- **Design Source**: `docs/design/real-simplex-consensus-wiring/` (flows, tests, strategy).
- **Design Contracts**:
  - Engine startup must use `CommonwareNetworkProvider::start_per_channel()` to get three P2P `(Sender, Receiver)` pairs and build vendor `simplex::Config` with signer/validator metadata.
  - `MailboxActor` must drive `ConsensusApp::{genesis, propose, verify}` and surface finalized height via `FinalizationSink` (no digest heuristics).
  - `whirlpool-node` must keep the `OracleHandle` alive, pass signer/validator data, and construct the engine with the real blocker.
  - Guardrails: No `vendor/` edits, no changes to `crates/consensus/` traits (engine/app/event pipeline stays as designed).
- **Current Implementation Anchors (grounding map)**:
  - `crates/consensus-simplex/src/engine.rs`: `CommonwareEngine::start()` is a stub that simulates finalization and never hits `simplex::Engine::start()`.
  - `crates/consensus-simplex/src/mailbox.rs`: `Mailbox` exists but implements `Relay::broadcast` as a no-op and the actor uses simplifications.
  - `crates/p2p-commonware/src/provider.rs`: `start_per_channel()` and Oracle handle plumbing are implemented and ready for wiring.
  - `crates/whirlpool-node/src/main.rs`: binary wiring needs updating to match the new constructor and keep the Oracle handle alive.

## Work Objectives
**Core Objective**: Transition the Simplex consensus wiring from simulated stubs to a real vendor-backed lifecycle while keeping observable test contracts aligned.
**Deliverables**:
- Failure-grounding tests for the stubbed engine path (Task 1 + Task 2).
- Real per-channel vendor wiring plus actor refinements (Tasks 3 + 4).
- Updated node binary wiring that honors the real engine builder and blocker (Task 5).
**Definition of Done**:
- `nix develop --command cargo build --workspace` passes.
- `nix develop --command cargo test --workspace` passes.
**Must Have**:
- TDD-first flow: failing tests captured before implementing fixes.
- CLI-based verification captured to evidence (`.sisyphus/evidence/task-N-slug.txt`).
**Must NOT Have**:
- No edits under `vendor/`.
- No changes to `crates/consensus/` trait definitions.

## Verification Strategy
### ZERO HUMAN INTERVENTION
All verification steps must run via the CLI commands listed in each task. Capture both failing (first) and passing runs to `.sisyphus/evidence/task-N-slug.txt` so reviewers can trace the TDD cycle.
**Evidence Convention**: Each task writes to `.sisyphus/evidence/task-N-slug.txt` (e.g., `.sisyphus/evidence/task-03-real-simplex-wiring.txt`). Record failing test output before implementation, then append the passing run once the fix is in place.

## Execution Strategy
### Parallel Execution Waves
- **Wave 1**: Task 1, Task 2.
- **Wave 2**: Task 03.1, Task 03.2, Task 03.3, Task 03.4 (decomposed from original Task 3). Core wiring depends on the failing tests already in place.
- **Wave 3**: Task 4, Task 5. Task 4 refines the Mailbox path, and Task 5 relies on the engine wiring, so they both target Wave 3. Both depend on Task 03.4 (last sub-task of decomposed Task 3).

### Dependency Matrix
| Task | Depends On | Wave |
|---|---|---|
| T1 | none | 1 |
| T2 | none | 1 |
| T03.1 | T1, T2 | 2 |
| T03.2 | T03.1 | 2 |
| T03.3 | T03.2 | 2 |
| T03.4 | T03.3 | 2 |
| T4 | T03.4 | 3 |
| T5 | T03.4 | 3 |

### Agent Dispatch Summary
| Task | Complexity | Category | Skills | Notes |
|---|---|---|---|---|
| T1 | S | quick | ctx-investigate | Harden unit tests around observable engine status/height.
| T2 | M | unspecified-low | ctx-investigate | Add single-validator integration tests that fail on stub.
| T03.1 | S | quick | ctx-investigate | Fix engine constructor tokio runtime panic in tests.
| T03.2 | S | quick | ctx-investigate | Change engine network generic to per-channel provider.
| T03.3 | M | unspecified-low | ctx-investigate | Wire `CommonwareEngine::start()` to vendor `simplex::Engine` via per-channel networking.
| T03.4 | M | unspecified-low | ctx-investigate | Align test names to AC and fix integration tests for real engine.
| T4 | M | unspecified-low | ctx-investigate | Polish `Mailbox`/`MailboxActor` flows so E2E tests pass.
| T5 | S | quick | ctx-investigate | Sync `whirlpool-node` wiring and validate workspace build/test.

## Task List
<!-- TASKS_START -->
- [ ] Task 1: Write Engine Unit Tests [**S**] → [tasks/01-engine-unit-tests.md](tasks/01-engine-unit-tests.md)
- [ ] Task 2: Write E2E Consensus Integration Tests [**M**] → [tasks/02-e2e-consensus-tests.md](tasks/02-e2e-consensus-tests.md)
- [ ] Task 03.1: Fix Engine Constructor and Test Infrastructure [**S**] → [tasks/03.1-fix-engine-constructor-tests.md](tasks/03.1-fix-engine-constructor-tests.md)
- [ ] Task 03.2: Change Engine Network Generic to Per-Channel Provider [**S**] → [tasks/03.2-engine-per-channel-network.md](tasks/03.2-engine-per-channel-network.md)
- [ ] Task 03.3: Wire Real Simplex Engine in start() [**M**] → [tasks/03.3-wire-real-simplex-engine.md](tasks/03.3-wire-real-simplex-engine.md)
- [ ] Task 03.4: Align Test Names and Fix Integration Tests [**M**] → [tasks/03.4-align-tests-integration.md](tasks/03.4-align-tests-integration.md)
- [ ] Task 4: Close Mailbox/MailboxActor Gaps [**M**] → [tasks/04-close-mailbox-gaps.md](tasks/04-close-mailbox-gaps.md)
- [ ] Task 5: Update whirlpool-node Wiring [**S**] → [tasks/05-node-wiring-update.md](tasks/05-node-wiring-update.md)

## Final Verification
After Task 5 completes, capture final proofs of the workspace-wide commands:
- `nix develop --command cargo build --workspace`
- `nix develop --command cargo test --workspace`
Record output to `.sisyphus/evidence/final-verification.txt`.
