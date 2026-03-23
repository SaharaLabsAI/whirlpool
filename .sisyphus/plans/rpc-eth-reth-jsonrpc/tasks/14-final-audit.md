# Task 14: Final audit and evidence reconciliation

## Status
- pending

## Dependencies
- 13

## Wave
- Wave 6

## Complexity
- S

## Target crates
- `rpc-eth` - audited implementation crate
- `whirlpool-node` - audited integration boundary
- `testing/integration-tests` - audited verification crate

## Pre-Task Gate
- [ ] Tasks 01 through 13 are complete.
- [ ] All required evidence files from earlier tasks exist.
- [ ] Artifact Registry rows for TST-1 through TST-12 have actual names/locations/statuses.
- [ ] This task is explicitly non-committing.
- [ ] Scope is limited to final verification and documentation reconciliation.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/e2e/rpc-eth-reth-jsonrpc-20260311-0522/scratch/agent/requirements.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/e2e/rpc-eth-reth-jsonrpc-20260311-0522/scratch/agent/tests.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
- Codebase references:
  - `.sisyphus/plans/rpc-eth-reth-jsonrpc/INDEX.md`
  - `.sisyphus/evidence/task-01-rpc-eth-reth-dependencies.md`
  - `.sisyphus/evidence/task-13-blob-and-remaining-rpc-integration-tests.md`

## What to do
1. Reconcile the Artifact Registry in `.sisyphus/plans/rpc-eth-reth-jsonrpc/INDEX.md` against the actual test names and locations produced by Tasks 02, 06, 07, 11, 12, and 13.
2. Rerun the full required verification matrix for `rpc-eth`, `whirlpool-node`, and the integration test target using `nix develop --command cargo ...` commands.
3. Confirm REQ-1 through REQ-7 and TST-1 through TST-12 are fully covered with no unresolved blockers.
4. Record any residual risk from `.whiteboard/rpc-eth-reth-jsonrpc/agent/blockers.md` that remains relevant after implementation.
5. Write the final audit summary to `.sisyphus/evidence/task-14-final-audit.md`.

## Mock Boundary
- None. This task validates the real implementation and previously-created tests.
- If a blocker remains, document it explicitly instead of masking it with stubs.

## AC trace
- REQ-1
- REQ-2
- REQ-3
- REQ-4
- REQ-5
- REQ-6
- REQ-7
- TST-1
- TST-2
- TST-3
- TST-4
- TST-5
- TST-6
- TST-7
- TST-8
- TST-9
- TST-10
- TST-11
- TST-12

## Must NOT do
- Do not introduce new feature work.
- Do not amend earlier commits.
- Do not modify `vendor/**`.

## Acceptance Criteria
- [ ] Artifact Registry matches actual tests and statuses.
- [ ] The full required verification matrix passes.
- [ ] Final audit evidence summarizes REQ/TST closure and any residual risk.
- [ ] The task remains non-committing and leaves the tree ready for user review.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo test -p rpc-eth`
- [ ] `nix develop --command cargo build -p whirlpool-node`
- [ ] `nix develop --command cargo test -p whirlpool-node`
- [ ] `nix develop --command cargo test -p integration-tests --test rpc_evm_integration`
- [ ] `nix develop --command cargo test -p integration-tests --test rpc_mem_integration`
- [ ] Evidence file exists and captures all command outcomes plus final REQ/TST reconciliation.

## Post-Task Reconciliation
- Mark every Artifact Registry row `done` or document the exact residual blocker in the status column.

## QA Scenarios
- Happy path: the full verification matrix passes and all REQ/TST rows reconcile cleanly.
- Failure path: one verification target regresses and must be recorded as a blocker before release.
- Boundary case: residual compile-time cost remains but no functional blocker is left.

## Evidence
- `.sisyphus/evidence/task-14-final-audit.md`
