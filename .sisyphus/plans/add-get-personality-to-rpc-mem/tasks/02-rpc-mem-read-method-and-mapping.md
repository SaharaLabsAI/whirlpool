# Task 02: Implement rpc-mem read method and deterministic response mapping

**Complexity**: M

## Summary
Implement `mem_getPersonality` in rpc-mem using finalized-storage-backed service reads and deterministic request/response mapping while preserving existing submit behavior.

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
No new external mocks. Reuse task-1 fake service adapters and rpc-mem internal error mapping.

## What to do
1. Expand rpc-mem service boundary to include read operations over decoded `personality_id` bytes.
2. Register `mem_getPersonality` method and wire request decode/validation.
3. Map found and not-found service outcomes to deterministic response payload contract.
4. Ensure malformed identity input maps to validation errors and does not call the service.
5. Run and fix task-1 tests until all pass.

## Acceptance Criteria
```bash
nix develop --command cargo test -p rpc-mem
```

## Evidence
- `.sisyphus/evidence/add-get-personality-to-rpc-mem/task-02-rpc-mem-read-impl.txt`

## Commit
Committing task. Do not advance until validation passes and a dedicated commit succeeds.
