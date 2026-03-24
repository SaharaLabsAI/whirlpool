# Add get personality to rpc-mem — Execution Plan

## TL;DR
- Add a read-capable `mem_getPersonality` surface in `rpc-mem` while preserving current submit behavior.
- Critical path: rpc-mem test contracts -> read service boundary and method wiring -> whirlpool-node adapter wiring -> integration verification.
- Parallelism is intentionally minimal because interface/contract steps must land before node wiring.

## Context
- Original request: add a personality read endpoint to rpc-mem using finalized storage semantics.
- Approved design: `.whiteboard/add-get-personality-to-rpc-mem/review/DESIGN.md`.
- Authoritative planning handoff: `.whiteboard/add-get-personality-to-rpc-mem/agent/handoff.md`, `.whiteboard/add-get-personality-to-rpc-mem/agent/TASK_GEN_READY.md`.
- Resolution: use finalized-only reads via `state::PersonalityStorage::get_latest` and keep `mem_submitPersonality` unchanged.

## Work Objectives
### Core Objective
Deliver an additive read RPC method for finalized personality retrieval with deterministic validation and response semantics, without regressing existing submit flow.

### Deliverables
- `rpc-mem` method registration and request/response contract for `mem_getPersonality`.
- `rpc-mem` service boundary expansion for read operations.
- `whirlpool-node` service adapter wiring for storage-backed reads.
- Regression and new tests covering `TST-1` through `TST-4`.

### Definition of Done
- `nix develop --command cargo build --workspace`
- `nix develop --command cargo test -p rpc-mem`
- `nix develop --command cargo test -p whirlpool-node`

### Must Have
- `REQ-1` through `REQ-7` covered by tasks.
- `TST-1` through `TST-4` covered by tasks.
- Each committing task is independently validated and commit-ready.

### Must NOT Have
- No pending/mempool read semantics for `mem_getPersonality`.
- No behavioral changes to `mem_submitPersonality`.
- No additional query/list endpoints beyond `personality_id` lookup.

## Verification Strategy
ZERO HUMAN INTERVENTION. Each task writes evidence under `.sisyphus/evidence/add-get-personality-to-rpc-mem/` and completes only after scoped validation commands pass in `nix develop`.

## Execution Strategy
### Parallel Execution Waves
- Wave 1: rpc-mem read contract tests and API surface.
- Wave 2: node wiring adapter integration.
- Wave 3: end-to-end verification and final audit.

### Dependency Matrix
- Task 1 -> Task 2 -> Task 3 -> Task 4

### Agent Dispatch Summary
- Keep tasks sequential because each task consumes interfaces and evidence produced by previous tasks.

## Task List
<!-- TASKS_START -->
- [x] Task 1: Define rpc-mem read contracts and behavior tests [**M**] -> [tasks/01-rpc-mem-read-contracts-and-tests.md](tasks/01-rpc-mem-read-contracts-and-tests.md)
- [x] Task 2: Implement rpc-mem read method and deterministic response mapping [**M**] -> [tasks/02-rpc-mem-read-method-and-mapping.md](tasks/02-rpc-mem-read-method-and-mapping.md)
- [x] Task 3: Wire whirlpool-node read-capable rpc-mem service adapter [**M**] -> [tasks/03-whirlpool-node-read-adapter-wiring.md](tasks/03-whirlpool-node-read-adapter-wiring.md)
- [x] Task 4: Integration verification and contract audit [**M**] -> [tasks/04-integration-verification-and-audit.md](tasks/04-integration-verification-and-audit.md)
<!-- TASKS_END -->

## Artifact Registry
<!-- ARTIFACTS_START -->
| TestID | Planned Name | Actual Name | Location | Created By | Status |
|--------|--------------|-------------|----------|------------|--------|
| TST-1 | RPC get personality returns latest finalized entry | `rpc_server_returns_latest_finalized_personality_entry`, `test_mem_get_personality_returns_finalized_entry_after_submit` | `crates/rpc-mem/tests/get_personality_contract.rs`, `testing/integration-tests/tests/rpc_mem_integration.rs` | Task 1, Task 3 | done |
| TST-2 | RPC get personality returns null/not-found when absent | `rpc_server_returns_null_when_personality_is_missing`, `test_mem_get_personality_returns_null_when_missing` | `crates/rpc-mem/tests/get_personality_contract.rs`, `testing/integration-tests/tests/rpc_mem_integration.rs` | Task 1, Task 3 | done |
| TST-3 | RPC get personality rejects malformed identity hex | `rpc_server_rejects_malformed_personality_hex_without_calling_service` | `crates/rpc-mem/tests/get_personality_contract.rs` | Task 1 | done |
| TST-4 | Submit path remains functional | `rpc_server_accepts_valid_submission`, `rpc_server_rejects_oversize_submission`, `test_mem_submit_personality_on_mem_rpc_only` | `crates/rpc-mem/tests/submit_regression.rs`, `testing/integration-tests/tests/rpc_mem_integration.rs` | Task 1, Task 3 | done |
<!-- ARTIFACTS_END -->

## Final Verification
Run the final audit task at [tasks/04-integration-verification-and-audit.md](tasks/04-integration-verification-and-audit.md).
