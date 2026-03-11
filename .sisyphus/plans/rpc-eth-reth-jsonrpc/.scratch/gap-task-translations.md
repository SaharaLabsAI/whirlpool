# Gap to Task Translation

## Task Map
- Task 01 (`rpc-eth`, interface/deps): add reth RPC/provider/pool/network dependencies and remove obsolete direct deps. Complexity S. Covers REQ-1, REQ-2, REQ-3, REQ-4.
- Task 02 (`rpc-eth`, implementation): create `src/provider.rs` with `WhirlpoolProvider`, stub traits, and provider-bound compile tests. Complexity L. Covers REQ-2, TST-1.
- Task 03 (`rpc-eth`, implementation): add real block/header/hash readers. Complexity M. Covers REQ-2, TST-1, TST-6, TST-8.
- Task 04 (`rpc-eth`, implementation): add state, transaction, and receipt readers plus state provider factory tests. Complexity M. Covers REQ-2, TST-1, TST-7, TST-9.
- Task 05 (`rpc-eth`, implementation): add chain-spec, account, node-primitives, and canonical subscription adapters. Complexity M. Covers REQ-2, REQ-4, TST-1.
- Task 06 (`rpc-eth`, implementation): create `src/pool.rs` and reject blob transactions. Complexity M. Covers REQ-3, REQ-5, TST-2, TST-9, TST-10.
- Task 07 (`rpc-eth`, implementation): create `src/network.rs` implementing `NetworkInfo`. Complexity S. Covers REQ-4, TST-3.
- Task 08 (`rpc-eth`, implementation): add `src/convert.rs` helpers for `EvmBlock` and raw tx conversion. Complexity M. Covers REQ-1, REQ-2, TST-8, TST-9.
- Task 09 (`rpc-eth`, implementation): rewrite `src/server.rs` to build the reth module stack and install blob unsupported handling. Complexity M. Covers REQ-1, REQ-5, TST-4, TST-5, TST-6, TST-10.
- Task 10 (`rpc-eth`, interface cleanup): rewrite `src/lib.rs`, define `RpcConfig`, remove legacy exports, and keep crate tests green. Complexity S. Covers REQ-1, REQ-6.
- Task 11 (`whirlpool-node`, integration): update node startup to construct `RpcConfig` and remove `ReceiptStore` / `EthRpcContext` usage. Complexity M. Covers REQ-6, TST-12.
- Task 12 (`testing/integration-tests`, behavior tests): author startup, chain ID, block number, and balance coverage mirroring reth HTTP patterns. Complexity L. Covers REQ-1, REQ-2, REQ-7, TST-4, TST-5, TST-6, TST-7.
- Task 13 (`testing/integration-tests`, behavior tests): add block retrieval, raw transaction, blob exclusion, and permutation coverage. Complexity M. Covers REQ-3, REQ-5, REQ-7, TST-8, TST-9, TST-10, TST-11.
- Task 14 (`workspace`, audit): reconcile evidence and rerun the required verification matrix. Complexity S. Covers REQ-1..REQ-7, TST-1..TST-12. Non-committing.

## Dependency Order
01 -> 02 -> 03 -> 04 -> 05 -> 06/07/08 -> 09 -> 10 -> 11 -> 12 -> 13 -> 14

## Wave Assignment
- Wave 1: 01
- Wave 2: 02-05
- Wave 3: 06-08
- Wave 4: 09-10
- Wave 5: 11-13
- Wave 6: 14

## Test References
- test_ref: { id: "TST-1", resolved_name: null, status: "to_be_created_by: Task 02" }
- test_ref: { id: "TST-2", resolved_name: null, status: "to_be_created_by: Task 06" }
- test_ref: { id: "TST-3", resolved_name: null, status: "to_be_created_by: Task 07" }
- test_ref: { id: "TST-4", resolved_name: null, status: "to_be_created_by: Task 12" }
- test_ref: { id: "TST-5", resolved_name: null, status: "to_be_created_by: Task 12" }
- test_ref: { id: "TST-6", resolved_name: null, status: "to_be_created_by: Task 12" }
- test_ref: { id: "TST-7", resolved_name: null, status: "to_be_created_by: Task 12" }
- test_ref: { id: "TST-8", resolved_name: null, status: "to_be_created_by: Task 13" }
- test_ref: { id: "TST-9", resolved_name: null, status: "to_be_created_by: Task 13" }
- test_ref: { id: "TST-10", resolved_name: null, status: "to_be_created_by: Task 13" }
- test_ref: { id: "TST-11", resolved_name: null, status: "to_be_created_by: Task 13" }
- test_ref: { id: "TST-12", resolved_name: null, status: "to_be_created_by: Task 11" }
