# Gap Task Translations

Derived from `design-contract-table.md` gap rows, with ordering constrained by:
1) interface-first, 2) TDD (tests before impl per crate), 3) build order from `proven-ac.md` (`app -> state -> app-evm -> state-reth -> rpc-eth -> whirlpool-node`).

## Wave Logic
- **Wave 1**: No dependencies
- **Wave 2**: Depends only on Wave 1
- **Wave 3**: Depends on Wave 1-2
- **Wave 4**: Depends on Wave 1-3
- **Wave 5**: Depends on Wave 1-4

### Task 01: app-receipt-reexport
- **Type**: interface
- **Crate(s)**: `app` (interface anchor for `Receipt` type)
- **Complexity**: S
- **Wave**: 1
- **Dependencies**: none
- **Description**: Re-export `alloy_consensus::Receipt` from `app` so downstream trait signatures and call sites use a shared receipt type.
- **Design Sources**: `CRATES.md` (app), `WORKSPACE.md` Phase 2, `docs/crates/app.md`
- **AC Coverage**: AC-3, AC-5 (type-level prerequisite for receipt persistence/capture)
- **Test IDs**: none (type re-export only)

### Task 02: state-blockstorage-trait-and-contract-check
- **Type**: interface
- **Crate(s)**: `state` (new trait boundary)
- **Complexity**: M
- **Wave**: 1
- **Dependencies**: none
- **Description**: Add and export `BlockStorage` trait (`store_block`, `get_block_by_number`, `get_block_by_hash`, `get_receipts_by_block`) and add compile-time/object-safety contract checks.
- **Design Sources**: `STRATEGY.md` Stream 1, `DOMAINS.md` Storage API, `docs/crates/state.md`
- **AC Coverage**: AC-1, AC-2, AC-3, AC-6, AC-9 (interface foundation)
- **Test IDs**: TC-ST-01

### Task 03: app-evm-receipt-lifecycle-tests
- **Type**: test
- **Crate(s)**: `app-evm` (unit tests)
- **Complexity**: M
- **Wave**: 2
- **Dependencies**: Task 01, Task 02
- **Description**: Add tests for propose-time receipt capture, finalization-time flush/clear, empty-receipt behavior, and persistence error mapping.
- **Design Sources**: `TESTS.md` app-evm section, `STRATEGY.md` Stream 2, `docs/crates/app-evm.md`
- **AC Coverage**: AC-4, AC-5
- **Test IDs**: TC-AE-01, TC-AE-02, TC-AE-03, TC-AE-04

### Task 04: app-evm-finalization-persistence-impl
- **Type**: impl
- **Crate(s)**: `app-evm` (execution/finalization integration)
- **Complexity**: M
- **Wave**: 2
- **Dependencies**: Task 01, Task 02, Task 03
- **Description**: Implement receipt lifecycle (`pending_receipts`), expose needed conversion visibility, and add `store_finalized_block` path to persist finalized blocks via `BlockStorage`.
- **Design Sources**: `STRATEGY.md` Stream 2, `FLOWS.md` Flow 1 steps 5-8, `docs/crates/app-evm.md`
- **AC Coverage**: AC-4, AC-5
- **Test IDs**: TC-AE-01, TC-AE-02, TC-AE-03, TC-AE-04

### Task 05: state-reth-blockstorage-tests
- **Type**: test
- **Crate(s)**: `state-reth` (MDBX unit test suite)
- **Complexity**: M
- **Wave**: 3
- **Dependencies**: Task 02, Task 04
- **Description**: Add tests for atomic persistence, block-by-number/hash round-trip, receipts-by-block, missing block/hash handling, and TxNumber continuity.
- **Design Sources**: `TESTS.md` state-reth section, `STRATEGY.md` Stream 1, `docs/crates/state-reth.md`
- **AC Coverage**: AC-1, AC-2, AC-3, AC-9, AC-10
- **Test IDs**: TC-SR-01, TC-SR-02, TC-SR-03, TC-SR-04, TC-SR-05, TC-SR-06, TC-SR-07, TC-SR-08

### Task 06: state-reth-mdbx-blockstorage-implementation
- **Type**: impl
- **Crate(s)**: `state-reth` (storage backend implementation)
- **Complexity**: L
- **Wave**: 3
- **Dependencies**: Task 02, Task 04, Task 05
- **Description**: Implement `BlockStorage` on `RethStateDb` with one-write-transaction atomicity across Headers/HeaderNumbers/BodyIndices/Transactions/Tx maps/Receipts, including number/hash/read reconstruction and receipt reads.
- **Design Sources**: `STRATEGY.md` Stream 1 + tx numbering decision, `FLOWS.md` Flow 1 step 10 + Flows 2-3 read paths, `docs/crates/state-reth.md`
- **AC Coverage**: AC-1, AC-2, AC-3, AC-6, AC-9, AC-10
- **Test IDs**: TC-SR-01, TC-SR-03, TC-SR-05, TC-SR-07, TC-SR-08

### Task 07: rpc-eth-block-endpoint-tests
- **Type**: test
- **Crate(s)**: `rpc-eth` (handler/context tests with mock storage)
- **Complexity**: M
- **Wave**: 3
- **Dependencies**: Task 02
- **Description**: Add endpoint tests for number/hash queries, tag behavior (`latest/finalized/safe/earliest/pending` policy), full-vs-hash responses, and conversion/decode failures.
- **Design Sources**: `TESTS.md` rpc-eth section, `FLOWS.md` Flows 2-3, `docs/crates/rpc-eth.md`
- **AC Coverage**: AC-6, AC-7, AC-8, AC-9, AC-10, AC-12
- **Test IDs**: TC-RPC-01, TC-RPC-02, TC-RPC-03, TC-RPC-04, TC-RPC-05, TC-RPC-06, TC-RPC-07, TC-RPC-08

### Task 08: rpc-eth-context-and-api-surface
- **Type**: impl
- **Crate(s)**: `rpc-eth` (API/context wiring)
- **Complexity**: M
- **Wave**: 4
- **Dependencies**: Task 02, Task 07
- **Description**: Extend Eth API surface with `eth_getBlockByNumber`/`eth_getBlockByHash` and add `block_storage` capability to `EthRpcContext` constructor/types.
- **Design Sources**: `STRATEGY.md` Stream 3, `CRATES.md` rpc-eth, `docs/crates/rpc-eth.md`
- **AC Coverage**: AC-6, AC-7, AC-9, AC-10, AC-11
- **Test IDs**: TC-RPC-01, TC-RPC-04, TC-RPC-05, TC-RPC-07

### Task 09: rpc-eth-handler-implementation-and-conversion
- **Type**: impl
- **Crate(s)**: `rpc-eth` (endpoint behavior + mapping)
- **Complexity**: M
- **Wave**: 4
- **Dependencies**: Task 04, Task 06, Task 07, Task 08
- **Description**: Implement endpoint handler logic (storage calls, tag resolution policy, error mapping) and EvmBlock-to-RPC block conversion honoring `full` semantics.
- **Design Sources**: `FLOWS.md` Flows 2-3, `STRATEGY.md` Stream 3, `docs/crates/rpc-eth.md`
- **AC Coverage**: AC-6, AC-7, AC-8, AC-9, AC-10, AC-12
- **Test IDs**: TC-RPC-02, TC-RPC-03, TC-RPC-04, TC-RPC-06, TC-RPC-08

### Task 10: whirlpool-node-persisting-finalization-wiring
- **Type**: wiring
- **Crate(s)**: `whirlpool-node` (node assembly), `app-evm` (sink callback consumer), `rpc-eth` (context input)
- **Complexity**: M
- **Wave**: 5
- **Dependencies**: Task 04, Task 06, Task 08, Task 09
- **Description**: Wire a persisting finalization sink path to call `store_finalized_block` on finalized events and pass shared `RethStateDb` into RPC context as block storage.
- **Design Sources**: `FLOWS.md` Flow 4, `WORKSPACE.md` integration points, `docs/crates/whirlpool-node.md`
- **AC Coverage**: AC-4, AC-11, AC-12
- **Test IDs**: TC-INT-02, TC-FLW-04

### Task 11: integration-propose-finalize-query-e2e
- **Type**: test
- **Crate(s)**: `testing/integration-tests` (primary), with real `whirlpool-node`, `app-evm`, `state-reth`, `rpc-eth`
- **Complexity**: M
- **Wave**: 5
- **Dependencies**: Task 06, Task 09, Task 10
- **Description**: Add end-to-end integration coverage for propose->finalize->persist->RPC query and verify node wiring and atomic storage behavior across the full stack.
- **Design Sources**: `TESTS.md` integration + cross-crate flow sections, `INTENT.md` SC-2..SC-5, `proven-ac.md`
- **AC Coverage**: AC-4, AC-6, AC-9, AC-11, AC-12
- **Test IDs**: TC-INT-01, TC-INT-02, TC-FLW-01, TC-FLW-02, TC-FLW-03, TC-FLW-04
