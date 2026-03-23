# Task 06: Integration verification and final audit

**Complexity**: M

## Summary
Close the plan with integration coverage, restart-volatility verification, and a final contract audit proving the implementation satisfies the approved design boundaries.

## Requirements
- REQ-1
- REQ-4
- REQ-5
- REQ-6
- REQ-7
- REQ-8
- REQ-9

## Tests
- TST-001
- TST-004
- TST-005
- TST-006
- TST-007

## Mock Boundary
Use integration-test harnesses and real crate wiring where practical; only mock external boundaries that are already mocked by the existing test harness.

## What to do
1. Add or finalize integration scenarios for mixed ingress happy path, mixed block preservation, finalization-only visibility, replacement semantics, and prototype volatility.
2. Reconcile the Artifact Registry in `INDEX.md` with actual test names and locations.
3. Run the full targeted workspace verification for all affected crates.
4. Confirm the implementation does not add retrieval RPC, durable personality storage, or pre-finalization visibility.
5. Write the final audit evidence file summarizing REQ/TST coverage and any accepted gaps.

## Acceptance Criteria
```bash
nix develop --command cargo test --workspace && nix develop --command cargo build --workspace
```

## Evidence
- `.sisyphus/evidence/add-mem-tx-support/task-06-final-audit.txt`

## Commit
Committing task. Do not advance until validation passes and a dedicated commit succeeds.
