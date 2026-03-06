# Task 04: app-evm-finalization-persistence

**Status**: pending
**Dependencies**: 01, 02, 03
**Wave**: 2
**Complexity**: M
**Target Crate(s)**: app-evm (role: impl)

## Pre-Task Gate
- `nix develop --command cargo build -p app` succeeds.
- `nix develop --command cargo build -p state` succeeds.
- Task 03 tests exist and fail (as expected).

## Context
The `app-evm` crate must implement the finalization persistence flow. This includes capturing receipts in `propose()` and persisting them alongside the finalized block data when `store_finalized_block()` is called by the node layer. This fulfills the core requirement for historical block data storage.

## What to do

### TDD Flow
1. Make `build_header_from_evm_block` public for cross-module access.
2. Add a `pending_receipts` field to the `EvmApplication` struct to buffer receipts from `propose()`.
3. Implement `store_finalized_block()` method using a `BlockStorage` provider.
4. Verify the tests from Task 03 now pass.

### Specific steps
1. Edit `crates/app-evm/src/executor.rs`:
   - Change `build_header_from_evm_block` visibility from `private` to `pub`.
   - Add `pending_receipts: Arc<Mutex<Option<Vec<Receipt>>>>` to the `EvmApplication` struct.
   - Update `propose()` to populate `pending_receipts` after decoding and executing transactions.
   - Add `pub fn store_finalized_block<B: BlockStorage>(&self, block: &EvmBlock, storage: &B) -> Result<(), EvmAppError>` method.
   - Ensure the method calls `storage.store_block(block, receipts)` and then clears `pending_receipts`.
2. Add necessary imports for `alloy_consensus::Receipt`, `state::BlockStorage`, and `std::sync::{Arc, Mutex}`.

## Mock Boundary
N/A (actual implementation)

## Must NOT do
- Do NOT change the `propose()` or `verify()` core logic except for receipt capture.
- Do NOT implement the actual `BlockStorage` for MDBX here (that's Task 06).

## References
- `.design-scratch/e2e/persistent-blocks-history-20260306-1500/scratch/plan/grounding-map.md`
- `docs/crates/app-evm.md`

## Acceptance Criteria
- `nix develop --command cargo test -p app-evm` succeeds (including TC-AE-01..04).
- `nix develop --command cargo build -p app-evm` succeeds.

## Post-Task Gate
- Command: `nix develop --command cargo test -p app-evm && nix develop --command cargo build -p app-evm`
- Expected: exit 0
- Max retries: 3

## Post-Task Reconciliation
- Update Artifact Registry in INDEX.md (TC-AE-01..04 status: created).

## QA Scenarios
- QA-4: Finalization triggers storage.
- QA-5: Receipts captured from propose.

## Evidence
`.sisyphus/evidence/task-04-app-evm-finalization-persistence.txt`
