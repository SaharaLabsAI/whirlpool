# integration-tests: E2E System Tests

## Summary
Workspace-level integration tests that exercise the full Whirlpool stack (consensus + p2p + RPC + state + EVM).
This crate contains no library or binary — only integration tests under `tests/`.

Location: `testing/integration-tests/`

## Dependency Boundaries
- `app`: application traits and tx source (`InMemoryTxPool`).
- `app-evm`: EVM application, `build_sahara_chain_spec()` for chain spec construction.
- `rpc-eth`: `RpcConfig`, `start_rpc_server` — Ethereum JSON-RPC server wiring.
- `state-reth`: `RethStateDb`, `open_state_db()` — persistent MDBX state for RPC tests.
- `reth-rpc-builder`: `RpcServerHandle` type (test server lifecycle).
- `alloy-primitives`: Ethereum types (Address, B256, U256).
- `serde_json`: JSON construction for raw HTTP RPC calls.
- `reqwest`: HTTP client for raw RPC calls.
- `tempfile`: `TempDir` for ephemeral MDBX databases.
- `tokio`: async runtime with test-util.

## Test Files

### `tests/rpc_integration.rs`
Uses a shared test harness:
- `start_test_rpc() -> TestRpcServer`: Creates ephemeral `TempDir` + `RethStateDb` + `ChainSpec` via `build_sahara_chain_spec()`, wires through `RpcConfig` + `start_rpc_server()`. Returns handle, address, and temp dir.
- `RecordingTxSource`: Test `TxSource` implementation that records submitted transactions for assertion.
- `rpc_call(addr, method, params) -> serde_json::Value`: HTTP POST helper for raw JSON-RPC calls.

**Tests (8 total):**
- `tst4_server_returns_chain_id` (TST-4/5): Verifies `eth_chainId` returns Sahara chain ID (`0x313532363334`).
- `tst5_latest_block_number` (TST-6): Verifies `eth_blockNumber` returns `0x0` on empty DB.
- `tst6_balance_query_returns_zero_for_empty_db` (TST-7): Verifies `eth_getBalance` returns `0x0` for unknown address.
- `tst7_eth_syncing_returns_false` (TST-7/12): Verifies `eth_syncing` returns `false`.
- `tst8_get_block_by_number` (TST-8): Verifies `eth_getBlockByNumber("0x0", false)` returns `null` on empty DB.
- `tst9_send_raw_transaction_acceptance_and_blob_rejection` (TST-9): Sends valid EIP-1559 tx (accepted) and blob Type-3 tx (rejected with pool error).
- `tst10_blob_base_fee_behavior` (TST-10): Verifies `eth_blobBaseFee` returns method-not-found or error (blob support excluded).
- `tst11_request_shape_permutations` (TST-11): Mirrors reth rpc-builder test patterns — tests HTTP request/response shape for multiple methods in a single test.

### `tests/multinode_consensus.rs`
End-to-end consensus test for a 4-node network using `whirlpool_node::node::start_node`.
- **TC-INT-05**: Verifies multi-node consensus. Starts 4 in-process nodes, waits for block height >= 1 via `eth_blockNumber` RPC calls (testing/integration-tests/tests/multinode_consensus.rs:19).
- Uses distinct ephemeral data directories, P2P ports, and RPC ports for each node.
- Validates that heights across nodes remain synchronized within 1 block.

## Running
```bash
# Run only RPC integration tests
nix develop --command cargo test -p integration-tests --test rpc_integration

# Run only consensus integration tests
nix develop --command cargo test -p integration-tests --test multinode_consensus

# Run all integration tests
nix develop --command cargo test -p integration-tests

# Run workspace tests excluding integration tests (fast iteration)
nix develop --command cargo test --workspace --exclude integration-tests
```
