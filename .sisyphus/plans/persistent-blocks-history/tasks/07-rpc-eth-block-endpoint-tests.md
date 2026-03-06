# Task 07: rpc-eth-block-endpoint-tests

**Status**: pending
**Dependencies**: 02
**Wave**: 3
**Complexity**: M
**Target Crate(s)**: rpc-eth (role: test)

## Pre-Task Gate
- `nix develop --command cargo build -p state` succeeds.

## Context
Exposing historical block data via RPC requires implementing `eth_getBlockByNumber` and `eth_getBlockByHash`. This task adds the unit tests for these endpoints using a mock `BlockStorage` to verify the logic for tag resolution (e.g., "latest", "earliest"), full block vs. hash-only responses, and error cases (e.g., unknown block number).

## What to do

### TDD Flow
1. Write failing tests for `eth_getBlockByNumber` and `eth_getBlockByHash`.
2. Verify resolving tags like "latest" and "finalized" to the context height.
3. Verify formatting of JSON-RPC responses for both `full=true` and `full=false`.
4. Verify tests fail to compile (expected until API surface is updated in Task 08).

### Specific steps
1. Edit `crates/rpc-eth/src/eth_handler.rs` and add tests:
   - `TC-RPC-01`: `eth_getBlockByNumber(999)` returns JSON-RPC `null`.
   - `TC-RPC-02`: `eth_getBlockByNumber(0, true)` returns full block JSON.
   - `TC-RPC-03`: `eth_getBlockByNumber(0, false)` returns block with transaction hashes only.
   - `TC-RPC-04`: `eth_getBlockByHash(hash, true)` returns full block JSON or `null`.
   - `TC-RPC-05`: Tag resolution for "latest" and "finalized" uses context height.
   - `TC-RPC-06`: Tag resolution for "pending" returns JSON-RPC `null` (MVP policy).
   - `TC-RPC-07`: Tag resolution for "earliest" resolves to number 0.
   - `TC-RPC-08`: Conversion failure (e.g., corrupt storage) returns JSON-RPC internal error.

## Mock Boundary
- Use a mock implementation of the `BlockStorage` trait for unit tests.

## Must NOT do
- Do NOT implement the actual RPC endpoints or handlers in this task.
- Do NOT modify the existing `EthApi` trait in `eth_api.rs`.

## References
- `.design-scratch/e2e/persistent-blocks-history-20260306-1500/docs/TESTS.md`
- `docs/crates/rpc-eth.md`

## Acceptance Criteria
- `nix develop --command cargo test -p rpc-eth` (tests should fail to compile or run, proving the need for Task 08).

## Post-Task Gate
- Command: `nix develop --command cargo test -p rpc-eth`
- Expected: exit 1 (proving tests fail as expected)
- Max retries: 1

## Post-Task Reconciliation
- Update Artifact Registry in INDEX.md (TC-RPC-01..08 status: pending_impl).

## QA Scenarios
- QA-6, QA-7, QA-8, QA-9, QA-10: RPC queries.

## Evidence
`.sisyphus/evidence/task-07-rpc-eth-block-endpoint-tests.txt`
