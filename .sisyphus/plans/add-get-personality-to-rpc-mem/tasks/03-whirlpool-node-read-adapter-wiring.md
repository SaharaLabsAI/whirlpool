# Task 03: Wire whirlpool-node read-capable rpc-mem service adapter

**Complexity**: M

## Summary
Update whirlpool-node composition so rpc-mem receives both submit and finalized-read capabilities through a storage-backed adapter, satisfying `REQ-2` and finalized-only semantics assumptions.

## Requirements
- REQ-2
- REQ-5

## Tests
- TST-1
- TST-2
- TST-4

## Mock Boundary
Use existing node integration harness and in-memory state adapters; no network mocks.

## What to do
1. Add/extend node-side rpc-mem adapter type that combines tx-source submit and personality storage reads.
2. Inject finalized personality storage handle when constructing rpc-mem service in whirlpool-node startup wiring.
3. Verify submit flow wiring remains unchanged while enabling read calls.
4. Add/update node wiring tests or integration checks to exercise both methods.

## Acceptance Criteria
```bash
nix develop --command cargo test -p whirlpool-node
```

## Evidence
- `.sisyphus/evidence/add-get-personality-to-rpc-mem/task-03-whirlpool-node-wiring.txt`

## Commit
Committing task. Do not advance until validation passes and a dedicated commit succeeds.
