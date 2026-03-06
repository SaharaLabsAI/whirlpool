# Task 11: integration-e2e-tests

**Status**: pending
**Dependencies**: 06, 09, 10
**Wave**: 5
**Complexity**: M
**Target Crate(s)**: (cross-crate) (role: test)

## Pre-Task Gate
- `nix develop --command cargo build` succeeds for the entire workspace.
- Node wiring from Task 10 is complete.

## Context
Integration tests are the final verification for the "Persistent Block Storage & History Queries" feature. This task adds the end-to-end tests that verify the full pipeline: proposing a block, finalized block persistence, and RPC retrieval using both numeric numbers and block tags. This ensures all components work together correctly.

## What to do

### TDD Flow
1. Write integration tests that simulate a Whirlpool node lifecycle.
2. Verify that proposed blocks are correctly stored in the MDBX database.
3. Verify that `eth_getBlockByNumber` and `eth_getBlockByHash` RPC calls return the correct block data.
4. Verify the tests pass on the fully wired node stack.

### Specific steps
1. Create or edit `testing/integration-tests/src/lib.rs`:
   - `TC-INT-01`: Propose a block, finalize it, and then check `state_db.get_block_by_number()` to verify it's persisted correctly with receipts.
   - `TC-INT-02`: Start the full node, propose and finalize blocks, and then use `jsonrpsee` client to call `eth_getBlockByNumber` and verify the JSON output.
   - `TC-FLOW-01`: Verify that sequentially proposed blocks increment `TxNumber` correctly across the whole stack.
   - `TC-FLOW-02`: Verify that `eth_getBlockByHash` correctly resolves via the `HeaderNumbers` lookup.
   - `TC-FLOW-03`: Verify that `eth_getBlockByNumber` correctly resolves the "latest" and "finalized" tags.
   - `TC-FLOW-04`: Verify that the node wiring correctly integrates all persistence and query paths.

## Mock Boundary
- Use the real `WhirlpoolEvmConfig`, `RethStateDb`, `EvmApplication`, and `EthApiHandler`.
- Mock only the P2P or external consensus components if needed for unit isolation.

## Must NOT do
- Do NOT add tests that rely on external networks or non-deterministic behavior.
- Do NOT modify any core crates (only the integration test crate).

## References
- `.design-scratch/e2e/persistent-blocks-history-20260306-1500/docs/TESTS.md`
- `persistent-blocks-history/proven-ac.md`

## Acceptance Criteria
- `nix develop --command cargo test -p integration-tests` succeeds (including TC-INT-01..02 and TC-FLOW-01..04).

## Post-Task Gate
- Command: `nix develop --command cargo test -p integration-tests`
- Expected: exit 0
- Max retries: 3

## Post-Task Reconciliation
- Update Artifact Registry in INDEX.md (TC-INT-01..02 status: created).

## QA Scenarios
- QA-4, QA-6, QA-9, QA-11, QA-12: Full stack verification.

## Evidence
`.sisyphus/evidence/task-11-integration-e2e-tests.txt`
