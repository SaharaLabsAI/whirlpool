# Task 05: state-reth-blockstorage-tests

**Status**: pending
**Dependencies**: 02, 04
**Wave**: 3
**Complexity**: M
**Target Crate(s)**: state-reth (role: test)

## Pre-Task Gate
- `nix develop --command cargo build -p state` succeeds.
- `nix develop --command cargo build -p app-evm` succeeds.

## Context
Persistent storage requires MDBX integration. Before implementing `BlockStorage` on `RethStateDb`, this task adds a comprehensive test suite that verifies atomic storage across all 8 required tables, reconstruction of blocks, and TxNumber continuity. This ensures the implementation from Task 06 meets all functional requirements.

## What to do

### TDD Flow
1. Write failing tests for `store_block` to verify it populates all 8 MDBX tables (Headers, Transactions, Receipts, etc.).
2. Write failing tests for `get_block_by_number` and `get_block_by_hash` to verify round-trip fidelity.
3. Write failing tests for `TxNumber` monotonicity across sequential block stores.
4. Verify tests fail to compile (expected until `BlockStorage` is implemented on `RethStateDb`).

### Specific steps
1. Edit `crates/state-reth/src/db.rs` (or create `crates/state-reth/src/block_storage.rs` for tests) and add:
   - `TC-SR-01`: Atomic persistence of 3 txs + receipts in a single MDBX transaction.
   - `TC-SR-02`: Transaction aborts on mismatched receipt count.
   - `TC-SR-03`: `get_block_by_number` with 100% field reconstruction.
   - `TC-SR-04`: `get_block_by_number(999)` returns `None`.
   - `TC-SR-05`: `get_block_by_hash` lookup via `HeaderNumbers`.
   - `TC-SR-06`: `get_block_by_hash` returns `None` for random hash.
   - `TC-SR-07`: Sequential block stores increment `TxNumber` correctly via `BlockBodyIndices`.
   - `TC-SR-08`: `get_receipts_by_block` returns block receipts in order.

## Mock Boundary
- Use a temporary MDBX database for testing (`tempfile` crate).

## Must NOT do
- Do NOT implement the actual `BlockStorage` logic in this task.
- Do NOT change existing `StateDb` trait tests in `db.rs`.

## References
- `.design-scratch/e2e/persistent-blocks-history-20260306-1500/docs/TESTS.md`
- `docs/crates/state-reth.md`

## Acceptance Criteria
- `nix develop --command cargo test -p state-reth` (tests should fail to compile or run, proving the need for Task 06).

## Post-Task Gate
- Command: `nix develop --command cargo test -p state-reth`
- Expected: exit 1 (proving tests fail as expected)
- Max retries: 1

## Post-Task Reconciliation
- Update Artifact Registry in INDEX.md (TC-SR-01..08 status: pending_impl).

## QA Scenarios
- QA-1: Header persistence.
- QA-2: Transaction persistence.
- QA-3: Receipt persistence.

## Evidence
`.sisyphus/evidence/task-05-state-reth-blockstorage-tests.txt`
