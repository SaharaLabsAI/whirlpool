# Task 04: Add prototype personality store and finalization flushing

**Complexity**: M

## Summary
Add the prototype in-memory personality store plus finalization-time write application so personality state becomes visible only after finalized blocks.

## Requirements
- REQ-6
- REQ-7
- REQ-9

## Tests
- TST-005
- TST-006

## Mock Boundary
Use store-local tests for replacement semantics and sink-level tests around finalized events; do not introduce durable storage in this task.

## What to do
1. Write tests for finalization-only visibility and last-finalized-write-wins semantics before implementing the store.
2. Add a dedicated personality storage trait/backend under `crates/state` or `crates/state-memory` per the approved design.
3. Extend finalization handling to apply derived mem writes only when blocks finalize.
4. Keep failure semantics aligned with current finalized block persistence logging behavior.
5. Ensure no pre-finalization reads can observe pending mem writes.

## Acceptance Criteria
```bash
nix develop --command cargo test -p state-memory && nix develop --command cargo test -p whirlpool-node
```

## Evidence
- `.sisyphus/evidence/add-mem-tx-support/task-04-finalization-store.txt`

## Commit
Committing task. Do not advance until validation passes and a dedicated commit succeeds.
