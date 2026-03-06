# Task 02: state-blockstorage-trait

**Status**: pending
**Dependencies**: none
**Wave**: 1
**Complexity**: M
**Target Crate(s)**: state (role: interface)

## Pre-Task Gate
N/A (Wave 1 task)

## Context
The `BlockStorage` trait is the core abstraction for persistent block storage. It enables both the `app-evm` crate to save finalized blocks and the `rpc-eth` crate to query historical data. This trait must be defined in the `state` crate to maintain a clean boundary between application logic and storage implementation.

## What to do

### TDD Flow
1. Define the `BlockStorage` trait with its 4 methods.
2. Add a `BlockStorageError` enum and associate it with the trait.
3. Write a contract check to ensure the trait is object-safe and implements `Send + Sync`.
4. Verify the trait and its bounds.

### Specific steps
1. Create `crates/state/src/block_storage.rs` with the `BlockStorage` trait containing:
   - `fn store_block(&self, block: &EvmBlock, receipts: &[Receipt]) -> Result<(), BlockStorageError>`
   - `fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>, BlockStorageError>`
   - `fn get_block_by_hash(&self, hash: B256) -> Result<Option<EvmBlock>, BlockStorageError>`
   - `fn get_receipts_by_block(&self, number: u64) -> Result<Option<Vec<Receipt>>, BlockStorageError>`
2. Edit `crates/state/src/lib.rs` and add `mod block_storage; pub use block_storage::{BlockStorage, BlockStorageError};`.
3. Add a unit test in `block_storage.rs` for `TC-ST-01` to check object-safety and trait bounds.

## Mock Boundary
N/A

## Must NOT do
- Do NOT add any MDBX-specific logic to the `state` crate.
- Do NOT modify the existing `StateDb` trait.

## References
- `.design-scratch/e2e/persistent-blocks-history-20260306-1500/docs/TESTS.md`
- `docs/crates/state.md`

## Acceptance Criteria
- `nix develop --command cargo test -p state` succeeds.
- `nix develop --command cargo build -p state` succeeds.

## Post-Task Gate
- Command: `nix develop --command cargo test -p state && nix develop --command cargo build -p state`
- Expected: exit 0
- Max retries: 3

## Post-Task Reconciliation
- Update Artifact Registry in INDEX.md (TC-ST-01 status: created).

## QA Scenarios
N/A

## Evidence
`.sisyphus/evidence/task-02-state-blockstorage-trait.txt`
