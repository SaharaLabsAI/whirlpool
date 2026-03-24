# Task 04: Integration verification and contract audit

**Complexity**: M

## Summary
Run full validation and audit REQ/TST traceability to confirm read RPC behavior, submit regression safety, and finalized-storage semantics before handoff.

## Requirements
- REQ-1
- REQ-2
- REQ-3
- REQ-4
- REQ-5
- REQ-6
- REQ-7

## Tests
- TST-1
- TST-2
- TST-3
- TST-4

## Mock Boundary
Non-committing audit of assembled implementation and tests; no new runtime mocks.

## What to do
1. Run workspace and targeted crate validation commands inside nix shell.
2. Verify REQ/TST coverage from task outputs and update audit evidence with exact test names/locations.
3. Reconcile Artifact Registry in `INDEX.md` with actual test names created in tasks 1-3.
4. Produce final audit summary documenting any accepted gaps (should be none).

## Acceptance Criteria
```bash
nix develop --command cargo build --workspace
nix develop --command cargo test -p rpc-mem
nix develop --command cargo test -p whirlpool-node
```

## Evidence
- `.sisyphus/evidence/add-get-personality-to-rpc-mem/task-04-final-audit.txt`

## Commit
Committing task. Do not advance until validation passes and a dedicated commit succeeds.
