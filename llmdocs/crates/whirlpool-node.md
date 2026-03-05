# whirlpool-node: EVM Consensus Binary

## Summary
`whirlpool-node` runs Commonware consensus with `EvmApplication` on Sahara Chain.

Location: `crates/whirlpool-node/`

## Dependency Boundaries
- `consensus`: core interface traits from `consensus::traits`.
- `consensus-simplex`: simplex adapter and engine.
- `app`: application adapter + tx source implementations (`InMemoryTxPool`).
- `app-evm`: EVM app implementation + `app_evm::traits::StateProvider`.
- `state`: `StateDb` trait and `StateError` (interface only).
- `state-reth`: `RethStateDb` implementation for persistent state storage.
- `state-memory`: `InMemoryStateDb` implementation (test code only).
- `p2p-commonware`: network provider bridge.
- `rpc-eth`: Ethereum JSON-RPC server (extracted from former `rpc/` module).

## Canonical Trait Imports Used by Node
- `consensus::traits::ConsensusEngine`
- `app_evm::traits::StateProvider`
- `state::traits::StateDb`
- `state_reth::RethStateDb`

## main.rs Wiring
1. Initialize runtime and `FinalizationSink`.
2. Build Commonware network provider.
3. Open `RethStateDb` at `DEFAULT_DB_PATH` via `state_reth::open_state_db`.
4. Build `WhirlpoolEvmConfig` and `EvmApplication<RethStateDb>`.
5. Provide `InMemoryTxPool` as tx source.
6. Wrap app with `ApplicationAdapter`, construct `CommonwareEngine`, call `start()`.
7. Initialize `EthRpcContext` and start JSON-RPC server on `RPC_BIND_ADDR` (via `rpc_eth`).

## RPC
The RPC implementation lives in the separate `rpc-eth` crate. See `llmdocs/crates/rpc-eth.md`.

## Import Migration Rule
Use canonical `::traits::` paths for interface types; avoid non-canonical crate-root trait imports.

## Integration Tests
File: `tests/rpc_integration.rs`

Tests use `start_test_rpc()` helper that creates `InMemoryTxPool`, `InMemoryStateDb`, `EthRpcContext` (chain_id 313_371), and starts a JSON-RPC server on a random port.

Covered RPC methods:
- `eth_chainId`, `eth_gasPrice`, `eth_getBalance`, `eth_getTransactionCount`, `eth_estimateGas`, `eth_getTransactionReceipt`, `eth_sendRawTransaction` (transfer).

The transfer test builds a signed `TxLegacy` via `alloy-consensus`/`alloy-signer-local`, EIP-2718 encodes it, sends via `send_raw_transaction`, and verifies both the returned tx hash and pool contents.
