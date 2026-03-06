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
Tests use a `start_test_rpc()` helper that creates `InMemoryTxPool`, `InMemoryStateDb`, `EthRpcContext` (chain_id 313_371), and starts a JSON-RPC server on a random port.

Covered RPC methods:
- `eth_chainId`, `eth_gasPrice`, `eth_getBalance`, `eth_getTransactionCount`, `eth_estimateGas`, `eth_getTransactionReceipt`, `eth_sendRawTransaction` (transfer).

The transfer test builds a signed `TxLegacy` via `alloy-consensus`/`alloy-signer-local`, EIP-2718 encodes it, sends via `send_raw_transaction`, and verifies both the returned tx hash and pool contents.

## Running
```bash
# Run only integration tests
nix develop --command cargo test -p integration-tests

# Run workspace tests excluding integration tests (fast iteration)
nix develop --command cargo test --workspace --exclude integration-tests
```
