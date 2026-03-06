# Persistent Block Storage & History Queries Index

## TL;DR
- **Summary**: Implement persistent MDBX-backed block storage for Whirlpool node and expose history query endpoints via RPC.
- **Deliverables**: `BlockStorage` trait, MDBX implementation for `RethStateDb`, receipt lifecycle management in `app-evm`, and new `eth_getBlockByNumber`/`eth_getBlockByHash` RPC handlers.
- **Effort Estimate**: 11 tasks across 5 waves.
- **Critical Path**: `state` trait -> `app-evm` capture -> `state-reth` MDBX impl -> `rpc-eth` handlers -> `whirlpool-node` wiring.

## Context
- **Design Source**: `.design-scratch/e2e/persistent-blocks-history-20260306-1500/`
- **Implementation Anchors**:
  - `crates/app/src/types.rs`: `EvmBlock` & `ExecutionResult` definitions.
  - `crates/state-reth/src/db.rs`: `RethStateDb` MDBX backend.
  - `crates/rpc-eth/src/context.rs`: `EthRpcContext` API container.
  - `crates/whirlpool-node/src/main.rs`: Node initialization and wiring.

## Work Objectives
- **Core Objective**: Ensure every finalized block is persisted to MDBX with all transactions and receipts, accessible via standard Ethereum JSON-RPC methods.
- **Definition of Done**: 
  - `nix develop --command cargo build` succeeds for the entire workspace.
  - `nix develop --command cargo test` passes for all changed crates.
  - All 11 tasks marked as completed.
- **Must NOT Have**:
  - Modifications to `vendor/` crates.
  - Changes to `consensus-simplex` core logic.
  - Non-atomic block storage (Headers/Receipts must commit together).

## Verification Strategy
- **Zero Human Intervention**: Automated test suite covering storage unit tests, RPC unit tests, and cross-crate integration flows.
- **Evidence Convention**: Verification outputs stored in `.sisyphus/evidence/task-NN-<slug>.txt`.

## Execution Strategy
- **Parallel Waves**:
  - Wave 1: Foundation (app re-exports, state traits)
  - Wave 2: Capture (app-evm receipt lifecycle)
  - Wave 3: Implementation & Tests (state-reth MDBX, rpc-eth mocks)
  - Wave 4: Integration (rpc-eth context/handlers)
  - Wave 5: Deployment (node wiring, e2e tests)

### Dependency Matrix
| Wave | Task | Dependencies |
|------|------|--------------|
| 1 | 01, 02 | None |
| 2 | 03, 04 | 01, 02 |
| 3 | 05, 06, 07 | 02, 04 |
| 4 | 08, 09 | 02, 04, 06, 07 |
| 5 | 10, 11 | 04, 06, 08, 09 |

### Agent Dispatch Summary
- **Category: interface**: Tasks 01, 02
- **Category: impl**: Tasks 04, 06, 08, 09
- **Category: test**: Tasks 03, 05, 07, 11
- **Category: wiring**: Task 10

## Task List
<!-- TASKS_START -->
- [ ] Task 01: app-receipt-reexport [**S**] → [tasks/01-app-receipt-reexport.md](tasks/01-app-receipt-reexport.md)
- [ ] Task 02: state-blockstorage-trait [**M**] → [tasks/02-state-blockstorage-trait.md](tasks/02-state-blockstorage-trait.md)
- [ ] Task 03: app-evm-receipt-lifecycle-tests [**M**] → [tasks/03-app-evm-receipt-lifecycle-tests.md](tasks/03-app-evm-receipt-lifecycle-tests.md)
- [ ] Task 04: app-evm-finalization-persistence [**M**] → [tasks/04-app-evm-finalization-persistence.md](tasks/04-app-evm-finalization-persistence.md)
- [ ] Task 05: state-reth-blockstorage-tests [**M**] → [tasks/05-state-reth-blockstorage-tests.md](tasks/05-state-reth-blockstorage-tests.md)
- [ ] Task 06: state-reth-mdbx-blockstorage [**L**] → [tasks/06-state-reth-mdbx-blockstorage.md](tasks/06-state-reth-mdbx-blockstorage.md)
- [ ] Task 07: rpc-eth-block-endpoint-tests [**M**] → [tasks/07-rpc-eth-block-endpoint-tests.md](tasks/07-rpc-eth-block-endpoint-tests.md)
- [ ] Task 08: rpc-eth-context-api-surface [**M**] → [tasks/08-rpc-eth-context-api-surface.md](tasks/08-rpc-eth-context-api-surface.md)
- [ ] Task 09: rpc-eth-handler-impl [**M**] → [tasks/09-rpc-eth-handler-impl.md](tasks/09-rpc-eth-handler-impl.md)
- [ ] Task 10: whirlpool-node-wiring [**M**] → [tasks/10-whirlpool-node-wiring.md](tasks/10-whirlpool-node-wiring.md)
- [ ] Task 11: integration-e2e-tests [**M**] → [tasks/11-integration-e2e-tests.md](tasks/11-integration-e2e-tests.md)
<!-- TASKS_END -->

## Artifact Registry
| TestID | Planned Name | Actual Name | Location | Created By | Status |
|--------|--------------|-------------|----------|------------|--------|
| TC-ST-01 | check_blockstorage_safety | (pending) | state/src/block_storage.rs | Task 02 | pending |
| TC-AE-01 | test_propose_receipt_capture | (pending) | app-evm/src/executor.rs | Task 03 | pending |
| TC-AE-02 | test_finalization_flush | (pending) | app-evm/src/executor.rs | Task 03 | pending |
| TC-AE-03 | test_finalization_no_receipts | (pending) | app-evm/src/executor.rs | Task 03 | pending |
| TC-AE-04 | test_persistence_error_handling | (pending) | app-evm/src/executor.rs | Task 03 | pending |
| TC-SR-01 | test_mdbx_atomic_storage | (pending) | state-reth/src/block_storage.rs | Task 05 | pending |
| TC-SR-02 | test_mdbx_mismatched_receipts | (pending) | state-reth/src/block_storage.rs | Task 05 | pending |
| TC-SR-03 | test_mdbx_get_block_number | (pending) | state-reth/src/block_storage.rs | Task 05 | pending |
| TC-SR-04 | test_mdbx_missing_block | (pending) | state-reth/src/block_storage.rs | Task 05 | pending |
| TC-SR-05 | test_mdbx_get_block_hash | (pending) | state-reth/src/block_storage.rs | Task 05 | pending |
| TC-SR-06 | test_mdbx_missing_hash | (pending) | state-reth/src/block_storage.rs | Task 05 | pending |
| TC-SR-07 | test_mdbx_tx_continuity | (pending) | state-reth/src/block_storage.rs | Task 05 | pending |
| TC-SR-08 | test_mdbx_get_receipts | (pending) | state-reth/src/block_storage.rs | Task 05 | pending |
| TC-RPC-01 | test_rpc_get_block_empty | (pending) | rpc-eth/src/eth_handler.rs | Task 07 | pending |
| TC-RPC-02 | test_rpc_get_block_full | (pending) | rpc-eth/src/eth_handler.rs | Task 07 | pending |
| TC-RPC-03 | test_rpc_get_block_hashes | (pending) | rpc-eth/src/eth_handler.rs | Task 07 | pending |
| TC-RPC-04 | test_rpc_get_block_by_hash | (pending) | rpc-eth/src/eth_handler.rs | Task 07 | pending |
| TC-RPC-05 | test_rpc_tag_resolution | (pending) | rpc-eth/src/eth_handler.rs | Task 07 | pending |
| TC-RPC-06 | test_rpc_pending_policy | (pending) | rpc-eth/src/eth_handler.rs | Task 07 | pending |
| TC-RPC-07 | test_rpc_earliest_tag | (pending) | rpc-eth/src/eth_handler.rs | Task 07 | pending |
| TC-RPC-08 | test_rpc_conversion_failure | (pending) | rpc-eth/src/eth_handler.rs | Task 07 | pending |
| TC-INT-01 | test_e2e_persistence | (pending) | testing/integration-tests/src/lib.rs | Task 11 | pending |
| TC-INT-02 | test_e2e_rpc_query | (pending) | testing/integration-tests/src/lib.rs | Task 11 | pending |

## Final Verification
- Run full workspace build: `nix develop --command cargo build`
- Run full workspace tests: `nix develop --command cargo test`
