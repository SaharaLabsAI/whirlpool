# Add mem tx support — Execution Plan

## TL;DR
- Deliver `crates/app-mem` and `crates/rpc-mem`, keep the shared mempool generic, and extend node wiring so finalized personality state is written only after finalization.
- Critical path: `app-mem` contract -> `rpc-mem` ingress -> mixed proposal/verification -> finalization store -> node dual-RPC wiring -> integration audit.
- Parallelism is limited because the design requires interface crate work before implementation wiring.

## Context
- Original request: add mem/personality transaction support using `.whiteboard/personality-markdown-tx/review/DESIGN.md` as a baseline, with an explicit `app-mem` crate.
- Approved design: `.whiteboard/add-mem-tx-support/review/DESIGN.md`.
- Authoritative planning handoff: `.whiteboard/add-mem-tx-support/agent/handoff.md`, `.whiteboard/add-mem-tx-support/agent/TASK_GEN_READY.md`.
- Resolution: keep `rpc-eth` Ethereum-only, preserve `TxSource` as opaque-byte ingress, and make prototype personality visibility finalization-only.

## Work Objectives
### Core Objective
Implement the approved prototype mem/personality transaction path without widening Ethereum RPC semantics or exposing pre-finalization personality state.

### Deliverables
- Workspace members for `crates/app-mem` and `crates/rpc-mem`.
- Deterministic mem payload codec/validation and finalized-write derivation.
- Mixed-family proposal/verification behavior that preserves EVM semantics.
- Prototype in-memory personality store and finalization-time flushing.
- Dual RPC startup from `crates/whirlpool-node`.
- Test coverage for `TST-001` through `TST-007`.

### Definition of Done
- `nix develop --command cargo build --workspace`
- `nix develop --command cargo test --workspace`
- Integration coverage exists for the approved mem/personality flows and guardrails.

### Must Have
- `REQ-1` through `REQ-9` covered by plan tasks.
- `TST-001` through `TST-007` covered by plan tasks.
- Per-task commit checkpoints for every committing task.

### Must NOT Have
- No merge of mem RPC into `rpc-eth`.
- No mempool split in v1.
- No durable personality storage or retrieval RPC in this plan.
- No implied Jolt-backed authenticity checks in v1.

## Verification Strategy
ZERO HUMAN INTERVENTION. Every task writes evidence under `.sisyphus/evidence/add-mem-tx-support/` and finishes only after its scoped validation passes inside `nix develop`.

## Execution Strategy
### Parallel Execution Waves
- Wave 1: foundation contracts and tests.
- Wave 2: RPC ingress and mixed execution path.
- Wave 3: finalization store and node wiring.
- Wave 4: integration and audit.

### Dependency Matrix
- Task 1 -> Task 2 -> Task 3 -> Task 4 -> Task 5 -> Task 6
- Task 6 is the final audit gate.

### Agent Dispatch Summary
- Keep tasks sequential because each later task consumes interfaces and evidence produced by the prior task.

## Task List
<!-- TASKS_START -->
- [x] Task 1: Add `app-mem` crate contracts and behavior tests [**M**] -> [tasks/01-app-mem-crate-and-tests.md](tasks/01-app-mem-crate-and-tests.md)
- [x] Task 2: Add `rpc-mem` ingress and submission tests [**M**] -> [tasks/02-rpc-mem-ingress.md](tasks/02-rpc-mem-ingress.md)
- [x] Task 3: Extend mixed proposal and verification without EVM regression [**L**] -> [tasks/03-mixed-proposal-verification.md](tasks/03-mixed-proposal-verification.md)
- [x] Task 4: Add prototype personality store and finalization flushing [**M**] -> [tasks/04-finalization-store.md](tasks/04-finalization-store.md)
- [x] Task 5: Wire workspace and dual RPC node composition [**M**] -> [tasks/05-workspace-and-node-wiring.md](tasks/05-workspace-and-node-wiring.md)
- [x] Task 6: Integration verification and final audit [**M**] -> [tasks/06-integration-and-audit.md](tasks/06-integration-and-audit.md)
<!-- TASKS_END -->

## Artifact Registry
<!-- ARTIFACTS_START -->
| TestID | Planned Name | Actual Name | Location | Created By | Status |
|--------|-------------|-------------|----------|------------|--------|
| TST-001 | Mixed ingress happy path | `submit_personality_enqueues_bytes_and_returns_hash`, `rpc_server_accepts_valid_submission` | `crates/rpc-mem/tests/submit_contract.rs` | Task 2 | done |
| TST-002 | Oversize markdown rejection | `rejects_oversize_markdown` | `crates/app-mem/src/lib.rs` | Task 1 | done |
| TST-003 | Hash mismatch rejection | `rejects_markdown_hash_mismatch` | `crates/app-mem/src/lib.rs` | Task 1 | done |
| TST-004 | Mixed block preservation | `propose_mixed_block_preserves_mem_and_executes_evm`, `verify_accepts_mixed_block_with_mem_transactions` | `crates/app-evm/src/executor.rs` | Task 3 | done |
| TST-005 | Finalization-only visibility | `store_is_empty_before_finalized_writes`, `prefinalized_events_do_not_make_personality_visible`, `finalized_events_persist_personality_writes` | `crates/state-memory/src/personality.rs`, `crates/whirlpool-node/src/persisting_sink.rs` | Task 4 | done |
| TST-006 | Replacement semantics | `latest_finalized_write_replaces_existing_personality` | `crates/state-memory/src/personality.rs` | Task 4 | done |
| TST-007 | Prototype volatility documentation | `in_memory_store_drops_state_across_instances` | `crates/state-memory/src/personality.rs` | Task 6 | done |
<!-- ARTIFACTS_END -->

## Final Verification
Run the final audit task at [tasks/06-integration-and-audit.md](tasks/06-integration-and-audit.md).
