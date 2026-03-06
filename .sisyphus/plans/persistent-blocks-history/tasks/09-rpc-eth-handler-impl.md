# Task 09: rpc-eth-handler-impl

**Status**: pending
**Dependencies**: 04, 06, 07, 08
**Wave**: 4
**Complexity**: M
**Target Crate(s)**: rpc-eth (role: impl)

## Pre-Task Gate
- `nix develop --command cargo build -p app-evm` succeeds.
- `nix develop --command cargo build -p state-reth` succeeds.
- Task 08 API surface changes exist.

## Context
Implementing the block query handlers in `rpc-eth` is the final step in providing historical block data to clients. This task requires a complex mapping from `EvmBlock` to `alloy_rpc_types::Block`, handling the `full` flag, and resolving `BlockNumberOrTag` to actual block numbers using the context's current height.

## What to do

### TDD Flow
1. Implement the `eth_getBlockByNumber` and `eth_getBlockByHash` handlers in `crates/rpc-eth/src/eth_handler.rs`.
2. Add a conversion function `evm_block_to_rpc_block` that handles full block and hash-only responses.
3. Implement `BlockNumberOrTag` resolution logic using the `EthRpcContext` height field.
4. Verify the tests from Task 07 now pass.

### Specific steps
1. Edit `crates/rpc-eth/src/eth_handler.rs`:
   - Update `eth_getBlockByNumber` handler to:
     - Resolve `BlockNumberOrTag` (latest, finalized, earliest, numeric).
     - Call `block_storage.get_block_by_number(number)`.
     - Convert the resulting `EvmBlock` to an `alloy_rpc_types::Block` response.
   - Update `eth_getBlockByHash` handler to:
     - Call `block_storage.get_block_by_hash(hash)`.
     - Convert the resulting `EvmBlock` to an `alloy_rpc_types::Block` response.
   - Add a private `evm_block_to_rpc_block` method that correctly formats the response for `full=true` and `full=false`.
2. Add necessary imports for `alloy_rpc_types`, `state::BlockStorage`, and `app::EvmBlock`.
3. Add internal error handling for conversion or storage failures.

## Mock Boundary
N/A (actual implementation)

## Must NOT do
- Do NOT modify the `BlockStorage` trait.
- Do NOT change the `EthRpcContext` generic types again.

## References
- `.design-scratch/e2e/persistent-blocks-history-20260306-1500/scratch/plan/grounding-map.md`
- `docs/crates/rpc-eth.md`

## Acceptance Criteria
- `nix develop --command cargo test -p rpc-eth` succeeds (including TC-RPC-01..08).
- `nix develop --command cargo build -p rpc-eth` succeeds.

## Post-Task Gate
- Command: `nix develop --command cargo test -p rpc-eth && nix develop --command cargo build -p rpc-eth`
- Expected: exit 0
- Max retries: 3

## Post-Task Reconciliation
- Update Artifact Registry in INDEX.md (TC-RPC-02, 03, 04, 06, 08 status: created).

## QA Scenarios
- QA-6, QA-7, QA-8, QA-9, QA-10: RPC queries.

## Evidence
`.sisyphus/evidence/task-09-rpc-eth-handler-impl.txt`
