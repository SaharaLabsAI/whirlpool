# Task 06: state-reth-mdbx-blockstorage

**Status**: pending
**Dependencies**: 02, 04, 05
**Wave**: 3
**Complexity**: L
**Target Crate(s)**: state-reth (role: impl)

## Pre-Task Gate
- `nix develop --command cargo build -p state` succeeds.
- `nix develop --command cargo build -p app-evm` succeeds.
- Task 05 tests exist and fail (as expected).

## Context
Implementing the `BlockStorage` trait for `RethStateDb` is the heart of the persistence feature. This task requires a complex mapping of `EvmBlock` fields (Headers, Transactions, Receipts) into the Reth MDBX schema, ensuring that all 8 required tables are updated atomically in a single write transaction and that TxNumbers are assigned monotonically.

## What to do

### TDD Flow
1. Implement the `BlockStorage` trait for `RethStateDb` in `crates/state-reth/src/block_storage.rs`.
2. Add table definitions and codec logic for `Headers`, `HeaderNumbers`, `BlockBodyIndices`, `Transactions`, `TransactionHashNumbers`, `TransactionBlocks`, `Receipts`, and `HeaderTerminalDifficulties`.
3. Implement `store_block`, `get_block_by_number`, `get_block_by_hash`, and `get_receipts_by_block` with atomic write/read semantics.
4. Verify the tests from Task 05 now pass.

### Specific steps
1. Create `crates/state-reth/src/block_storage.rs`:
   - Define a `store_block` implementation that:
     - Starts a write transaction on the Reth MDBX database.
     - Writes the block header and difficulty.
     - Maps the block hash to the block number in `HeaderNumbers`.
     - Assigns monotonic `TxNumber`s to each transaction by reading the end of the previous `BlockBodyIndices` entry.
     - Encodes and writes transactions, mapping hashes to `TxNumber`s.
     - Encodes and writes receipts to the `Receipts` table.
     - Commits the transaction or aborts on error.
   - Define `get_block_by_number` and `get_block_by_hash` handlers that read from the tables and reconstruct the `EvmBlock` with full fidelity.
2. Edit `crates/state-reth/src/lib.rs` and add `mod block_storage;`.
3. Ensure all MDBX table access uses existing Reth database APIs (e.g., `reth_db::DatabaseEnv`).

## Mock Boundary
N/A (actual implementation)

## Must NOT do
- Do NOT modify the `StateDb` implementation in `db.rs` unless required for shared state access.
- Do NOT use raw MDBX calls if the Reth database layer provides appropriate abstractions.

## References
- `.design-scratch/e2e/persistent-blocks-history-20260306-1500/scratch/plan/grounding-map.md`
- `docs/crates/state-reth.md`

## Acceptance Criteria
- `nix develop --command cargo test -p state-reth` succeeds (including TC-SR-01..08).
- `nix develop --command cargo build -p state-reth` succeeds.

## Post-Task Gate
- Command: `nix develop --command cargo test -p state-reth && nix develop --command cargo build -p state-reth`
- Expected: exit 0
- Max retries: 3

## Post-Task Reconciliation
- Update Artifact Registry in INDEX.md (TC-SR-01..08 status: created).

## QA Scenarios
- QA-1, QA-2, QA-3: Block artifact persistence.
- QA-9: getBlockByHash.

## Evidence
`.sisyphus/evidence/task-06-state-reth-mdbx-blockstorage.txt`
