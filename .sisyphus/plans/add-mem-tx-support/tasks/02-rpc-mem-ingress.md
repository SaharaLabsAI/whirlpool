# Task 02: Add rpc-mem ingress and submission tests

**Complexity**: M

## Summary
Create `crates/rpc-mem` with `mem_submitPersonality`, admission checks aligned with `app-mem`, and shared-ingress submission through `TxSource` without widening `rpc-eth` semantics.

## Requirements
- REQ-1
- REQ-2
- REQ-4
- REQ-8

## Tests
- TST-001
- TST-002

## Mock Boundary
Use a lightweight `TxSource` test double or in-memory implementation to verify accepted requests enqueue bytes and rejected requests do not mutate the shared ingress.

## What to do
1. Write request-level tests for successful submission and oversize rejection before implementing the RPC surface.
2. Create `crates/rpc-mem` with request types, server/bootstrap code, and a submission adapter targeting `app::traits::TxSource`.
3. Reuse `app-mem` validation and canonical encoding rules so admission and consensus-visible validation stay aligned.
4. Return deterministic tx hashes on accepted requests.
5. Keep the crate submit-only in v1.

## Acceptance Criteria
```bash
nix develop --command cargo test -p rpc-mem
```

## Evidence
- `.sisyphus/evidence/add-mem-tx-support/task-02-rpc-mem.txt`

## Commit
Committing task. Do not advance until validation passes and a dedicated commit succeeds.
