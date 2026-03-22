# Task 01: Add app-mem crate contracts and behavior tests

**Complexity**: M

## Summary
Create `crates/app-mem` and establish the canonical mem/personality payload model, deterministic structural validation, and finalized-write derivation contract required by `REQ-3`, `REQ-5`, `REQ-8`, and `REQ-9`.

## Requirements
- REQ-3
- REQ-5
- REQ-8
- REQ-9

## Tests
- TST-002
- TST-003

## Mock Boundary
No external mocks; use unit tests around pure payload encode/decode and validation functions.

## What to do
1. Add behavior-first tests covering oversize markdown rejection and markdown-hash mismatch rejection in the new `crates/app-mem` crate.
2. Create `crates/app-mem` with payload types, canonical encode/decode helpers, structural validation, and finalized personality-write derivation.
3. Keep cryptographic verification explicitly out of scope while preserving fields and APIs that allow later hardening.
4. Export the public API needed by `rpc-mem`, `app-evm`, and `whirlpool-node`.
5. Update workspace manifests as needed for the new crate.

## Acceptance Criteria
```bash
nix develop --command cargo test -p app-mem
```

## Evidence
- `.sisyphus/evidence/add-mem-tx-support/task-01-app-mem.txt`

## Commit
Committing task. Do not advance until validation passes and a dedicated commit succeeds.
