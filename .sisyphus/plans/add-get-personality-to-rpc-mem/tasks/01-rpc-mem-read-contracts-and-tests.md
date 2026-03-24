# Task 01: Define rpc-mem read contracts and behavior tests

**Complexity**: M

## Summary
Add behavior-first tests and contract fixtures for `mem_getPersonality` and submit regression so the read feature is test-defined before implementation, covering `REQ-1`, `REQ-3`, `REQ-4`, `REQ-6`, and `REQ-7`.

## Requirements
- REQ-1
- REQ-3
- REQ-4
- REQ-6
- REQ-7

## Tests
- TST-1
- TST-2
- TST-3
- TST-4

## Mock Boundary
Use in-crate fake service/storage adapters in rpc-mem tests; avoid external process mocks.

## What to do
1. Add behavior tests in `crates/rpc-mem/tests/` for found, not-found, malformed-hex, and submit-regression flows.
2. Define request/response fixtures expected by the tests, including deterministic binary field encoding assertions.
3. Add pre-implementation scaffolding only as needed to compile tests (types/trait signatures), without implementing read behavior logic.
4. Record artifact registry planned names and expected locations in the plan evidence.

## Acceptance Criteria
```bash
nix develop --command cargo test -p rpc-mem --test get_personality_contract
nix develop --command cargo test -p rpc-mem --test submit_regression
```

## Evidence
- `.sisyphus/evidence/add-get-personality-to-rpc-mem/task-01-rpc-mem-read-contracts.txt`

## Commit
Committing task. Do not advance until validation passes and a dedicated commit succeeds.
