# p2p-provider-completeness - Execution Plan

## TL;DR

| Field | Value |
|-------|-------|
| **Summary** | Complete Sub-Intent A by fixing validator seeding, bootstrap peer threading, and channel metadata preservation in the Commonware-backed P2P provider path |
| **Deliverables** | Updated plan tasks for `crates/p2p-commonware/src/{receiver.rs,provider.rs,lib.rs,sender.rs,traits.rs}` and `crates/whirlpool-node/src/main.rs`, plus final verification coverage for `REQ-1`, `REQ-2`, and `REQ-3` |
| **Effort** | 6 tasks, 6 waves, estimated M complexity overall |
| **Parallel** | none; `agent/handoff.md` requires strict serial execution |
| **Critical Path** | `01` -> `02` -> `03` -> `04` -> `05` -> `06` |

## Context

### Original Request

Generate an executable `.sisyphus` plan for Sub-Intent A: P2P Provider Completeness using the design docs under `docs/design/p2p-node-connectivity/agent/` and keep the plan scoped to `REQ-1`, `REQ-2`, and `REQ-3`.

### Review Summary

- `agent/TASK_GEN_READY.md` is `READY` with `ready_for_task_generation: true`.
- `agent/handoff.md` is the primary ordering authority and requires work in this order: `receiver.rs` -> `provider.rs` -> `lib.rs` -> `sender.rs` + `traits.rs` -> `main.rs` -> tests.
- `agent/crate-contracts/p2p-commonware.md` and `agent/crate-contracts/whirlpool-node.md` restrict source edits to `crates/p2p-commonware/src/{provider.rs,receiver.rs,sender.rs,lib.rs,traits.rs}` and `crates/whirlpool-node/src/main.rs`.
- `crates/p2p/**`, vendor code, Sub-Intent B/C work, and design doc updates remain out of scope.

### Resolution Notes

- `agent/strategy.md` presents a broader sequence that starts with `provider.rs`, but `agent/handoff.md` is the planner-facing authority and therefore drives the final task order.
- The proof document uses `AC-*` and `INV-*` identifiers for traceability only; all executable task files use stable `REQ-*` and `TST-*` references as required.

## Work Objectives

### Core Objective

Produce an implementation plan that fixes the three documented Sub-Intent A bugs without widening scope: seed validators during provider build (`REQ-1`), preserve bootstrap peers into discovery (`REQ-2`), and carry real channel metadata through the receive path (`REQ-3`).

### Deliverables

- Entry file: `.sisyphus/plans/p2p-provider-completeness.md`
- Plan index: `.sisyphus/plans/p2p-provider-completeness/INDEX.md`
- Task files under `.sisyphus/plans/p2p-provider-completeness/tasks/`
- Per-task evidence log targets under `.sisyphus/plans/p2p-provider-completeness/evidence/`

### Definition of Done

```bash
nix develop --command cargo build
nix develop --command cargo test
```

### Must Have

- Strict adherence to the handoff execution order.
- Explicit dependency declarations between tasks.
- Task-local file lists using exact crate-contract paths.
- Stable traceability through `REQ-1`, `REQ-2`, `REQ-3` and `TST-REQ1-001/002`, `TST-REQ2-001/002`, `TST-REQ3-001/002/003`.
- Verification commands that always use `nix develop --command`.
- A final integration task that runs the full test suite for this change set.

### Must NOT Have

- No source changes outside `crates/p2p-commonware/src/{provider.rs,receiver.rs,sender.rs,lib.rs,traits.rs}` and `crates/whirlpool-node/src/main.rs`.
- No modifications to `crates/p2p/**`, `vendor/**`, design docs, or `e2e-state.md`.
- No tasks for Sub-Intent B or Sub-Intent C.
- No `AC-*` references inside task execution instructions.
- No assumption that `cargo` is available without `nix develop --command`.

## Verification Strategy

ZERO HUMAN INTERVENTION: every task ends with command-driven acceptance checks and an evidence target under `.sisyphus/plans/p2p-provider-completeness/evidence/`. Per-task compile/test commands stay focused on the touched crate, while the final audit task runs the full workspace build and test commands required by the request.

## Execution Strategy

### Parallel Execution Waves

- Wave 1: Task 01 only
- Wave 2: Task 02 only
- Wave 3: Task 03 only
- Wave 4: Task 04 only
- Wave 5: Task 05 only
- Wave 6: Task 06 only

### Dependency Matrix

| Task | Depends On | Wave |
|------|------------|------|
| 01-receiver-channel-contract | none | 1 |
| 02-provider-build-seeding-and-bootstrap | 01-receiver-channel-contract | 2 |
| 03-multiplex-receiver-alignment | 01-receiver-channel-contract, 02-provider-build-seeding-and-bootstrap | 3 |
| 04-sender-traits-compatibility-review | 02-provider-build-seeding-and-bootstrap, 03-multiplex-receiver-alignment | 4 |
| 05-whirlpool-node-builder-wiring | 02-provider-build-seeding-and-bootstrap, 04-sender-traits-compatibility-review | 5 |
| 06-final-subintent-a-verification | 01-receiver-channel-contract, 02-provider-build-seeding-and-bootstrap, 03-multiplex-receiver-alignment, 04-sender-traits-compatibility-review, 05-whirlpool-node-builder-wiring | 6 |

### Agent Dispatch Summary

| Wave | Tasks | Parallel | Estimated Time |
|------|-------|----------|----------------|
| 1 | 01 | no | 10-15 min |
| 2 | 02 | no | 20-30 min |
| 3 | 03 | no | 10-15 min |
| 4 | 04 | no | 5-10 min |
| 5 | 05 | no | 10-20 min |
| 6 | 06 | no | 15-25 min |

## Task List

<!-- TASKS_START -->
- [x] Task 1: Receiver channel contract [**S**] -> [tasks/01-receiver-channel-contract.md](tasks/01-receiver-channel-contract.md)
- [x] Task 2: Provider build seeding and bootstrap [**M**] -> [tasks/02-provider-build-seeding-and-bootstrap.md](tasks/02-provider-build-seeding-and-bootstrap.md)
- [x] Task 3: Multiplex receiver alignment [**S**] -> [tasks/03-multiplex-receiver-alignment.md](tasks/03-multiplex-receiver-alignment.md)
- [x] Task 4: Sender and traits compatibility review [**S**] -> [tasks/04-sender-traits-compatibility-review.md](tasks/04-sender-traits-compatibility-review.md)
- [ ] Task 5: whirlpool-node builder wiring [**M**] -> [tasks/05-whirlpool-node-builder-wiring.md](tasks/05-whirlpool-node-builder-wiring.md)
- [ ] Task 6: Final Sub-Intent A verification [**M**] -> [tasks/06-final-subintent-a-verification.md](tasks/06-final-subintent-a-verification.md)
<!-- TASKS_END -->

## Artifact Registry

| Artifact | Purpose | Path |
|----------|---------|------|
| Task 01 evidence | Receiver channel contract verification | `.sisyphus/plans/p2p-provider-completeness/evidence/01-receiver-channel-contract.log` |
| Task 02 evidence | Provider validator/bootstrap verification | `.sisyphus/plans/p2p-provider-completeness/evidence/02-provider-build-seeding-and-bootstrap.log` |
| Task 03 evidence | Multiplex receiver verification | `.sisyphus/plans/p2p-provider-completeness/evidence/03-multiplex-receiver-alignment.log` |
| Task 04 evidence | Compatibility review verification | `.sisyphus/plans/p2p-provider-completeness/evidence/04-sender-traits-compatibility-review.log` |
| Task 05 evidence | Node wiring verification | `.sisyphus/plans/p2p-provider-completeness/evidence/05-whirlpool-node-builder-wiring.log` |
| Task 06 evidence | Final build and test verification | `.sisyphus/plans/p2p-provider-completeness/evidence/06-final-subintent-a-verification.log` |
| `TST-REQ1-001` | Provider build seeds non-empty validator set | `crates/p2p-commonware/src/provider.rs` |
| `TST-REQ1-002` | Empty validator set skips seeding | `crates/p2p-commonware/src/provider.rs` |
| `TST-REQ2-001` | Builder threads supplied bootstrappers into discovery config | `crates/p2p-commonware/src/provider.rs` |
| `TST-REQ2-002` | Node startup wiring populates builder bootstrappers and validators together | `crates/whirlpool-node/src/main.rs`, `crates/p2p-commonware/src/provider.rs` |
| `TST-REQ3-001` | Receiver emits configured vote channel | `crates/p2p-commonware/src/receiver.rs`, `crates/p2p-commonware/src/provider.rs` |
| `TST-REQ3-002` | Receiver emits configured certificate and resolver channels distinctly | `crates/p2p-commonware/src/receiver.rs`, `crates/p2p-commonware/src/provider.rs` |
| `TST-REQ3-003` | Multiplex receiver forwards already-tagged message without repair logic | `crates/p2p-commonware/src/lib.rs`, `crates/p2p-commonware/src/receiver.rs` |

## Final Verification

See [tasks/06-final-subintent-a-verification.md](tasks/06-final-subintent-a-verification.md) for the final audit task, including required `nix develop --command cargo build` and `nix develop --command cargo test` coverage.
