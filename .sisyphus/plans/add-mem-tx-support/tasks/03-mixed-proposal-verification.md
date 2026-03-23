# Task 03: Extend mixed proposal and verification without EVM regression

**Complexity**: L

## Summary
Update the mixed transaction path so proposal and verification classify raw bytes as EVM, mem, or invalid while preserving existing EVM execution behavior.

## Requirements
- REQ-4
- REQ-5
- REQ-8

## Tests
- TST-003
- TST-004

## Mock Boundary
Use existing `app-evm` test patterns plus targeted mixed-family fixtures. Avoid mocking the EVM executor beyond existing local test seams.

## What to do
1. Add behavior tests that cover hash-mismatch rejection and mixed block preservation before changing execution logic.
2. Introduce a classification path that delegates mem payload validation to `app-mem` and keeps EVM decoding/execution intact for EVM txs.
3. Ensure invalid mem payloads are rejected deterministically in both proposal and verification.
4. Preserve current EVM receipt/state-root behavior for valid EVM transactions.
5. Update or add integration tests proving mixed-family handling does not regress the current EVM path.

## Acceptance Criteria
```bash
nix develop --command cargo test -p app-evm
```

## Evidence
- `.sisyphus/evidence/add-mem-tx-support/task-03-mixed-proposal.txt`

## Commit
Committing task. Do not advance until validation passes and a dedicated commit succeeds.
