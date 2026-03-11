# integration-tests: E2E System Tests

## Summary
Workspace-level integration tests that exercise the full Whirlpool stack (consensus + p2p + RPC + state + EVM).
This crate contains no library or binary — only integration tests under `tests/`.

Location: `testing/integration-tests/`

## Dependency Boundaries
- `app`: application traits and tx source (`InMemoryTxPool`).
- `rpc-eth`: Ethereum JSON-RPC server wiring.
- `state-memory`: `InMemoryStateDb` for in-memory test state.
- `revm`: EVM account types (`AccountInfo`).
- `alloy-*`: Transaction construction, signing, provider, RPC types.
- `jsonrpsee`: RPC server (test harness).
- `tokio`: async runtime with test-util.
- `reqwest`: HTTP client for raw RPC calls.

## Test Files

### `tests/rpc_integration.rs`
Tests use two primary helpers:
- `start_test_rpc()`: creates `InMemoryTxPool`, `InMemoryStateDb`, and `NullBlockStorage` for legacy tests.
- `start_test_rpc_with_reth_storage()`: creates a real `RethStateDb` (temp file) for block history e2e tests.

Covered RPC methods:
- `eth_chainId`, `eth_gasPrice`, `eth_getBalance`, `eth_getTransactionCount`, `eth_estimateGas`, `eth_getTransactionReceipt`, `eth_sendRawTransaction`, `eth_blockNumber`.
- `eth_getBlockByNumber`, `eth_getBlockByHash`.

### `tests/multinode_consensus.rs`
End-to-end consensus test for a 4-node network using `whirlpool_node::node::start_node`.
- **TC-INT-05**: Verifies multi-node consensus. Starts 4 in-process nodes, waits for block height >= 1 via `eth_blockNumber` RPC calls (testing/integration-tests/tests/multinode_consensus.rs:19).
- Uses distinct ephemeral data directories, P2P ports, and RPC ports for each node.
- Validates that heights across nodes remain synchronized within 1 block.

Block History Tests:
- `TC-INT-01`: Store and retrieve block via RPC (latest tag). Verifies field mapping and height synchronization.
- `TC-INT-02`: `eth_getBlockByHash` round-trip. Verifies lookup by block hash in `RethStateDb`.
- `TC-INT-03`: Missing block returns null. Verifies error handling for unknown block numbers.
- `TC-INT-04`: Multiple sequential blocks. Verifies that multiple blocks can be persisted and retrieved independently.

The transfer test builds a signed `TxLegacy` via `alloy-consensus`/`alloy-signer-local`, EIP-2718 encodes it, sends via `send_raw_transaction`, and verifies both the returned tx hash and pool contents.

## Running
```bash
# Run only integration tests
nix develop --command cargo test -p integration-tests

# Run workspace tests excluding integration tests (fast iteration)
nix develop --command cargo test --workspace --exclude integration-tests
```
