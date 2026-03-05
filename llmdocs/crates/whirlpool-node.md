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
- `state-memory`: `InMemoryStateDb` implementation for state storage.
- `p2p-commonware`: network provider bridge.
- `jsonrpsee`: JSON-RPC server framework (0.26.0).
- `alloy-rpc-types`: Ethereum RPC types (1.4.3).

## Canonical Trait Imports Used by Node
- `consensus::traits::ConsensusEngine`
- `app_evm::traits::StateProvider`
- `state::traits::StateDb`

## main.rs Wiring
1. Initialize runtime and `FinalizationSink`.
2. Build Commonware network provider.
3. Construct `TestStateDb(InMemoryStateDb)` implementing `StateDb`, `StateProvider`, and `revm::Database`.
4. Build `WhirlpoolEvmConfig` and `EvmApplication`.
5. Provide `InMemoryTxPool` as tx source.
6. Wrap app with `ApplicationAdapter`, construct `CommonwareEngine`, call `start()`.
7. Initialize `EthRpcContext` and start JSON-RPC server on `RPC_BIND_ADDR`.

## RPC Module (`rpc/`)
- `context.rs`: `EthRpcContext` holds shared `Arc` handles to `InMemoryTxPool`, `StateDb`, and `ReceiptStore`.
- `eth_api.rs`: `EthApi` trait defining 7 supported `eth_*` methods.
- `eth_handler.rs`: `EthApiHandler` implements `EthApiServer` for `EthApi`.
- `receipt_store.rs`: `ReceiptStore` providing in-memory mapping of transaction hash to `TransactionReceipt`.
- `server.rs`: `start_rpc_server` entry point using `jsonrpsee`.

### Supported RPC Methods
- `eth_chainId`: returns the configured Sahara chain ID.
- `eth_gasPrice`: returns hardcoded 1 gwei (v1).
- `eth_getBalance`: returns balance from `StateDb` for "latest" block.
- `eth_getTransactionCount`: returns nonce from `StateDb` for "latest" block.
- `eth_sendRawTransaction`: pushes raw bytes directly into `InMemoryTxPool`.
- `eth_estimateGas`: returns hardcoded 21,000 gas (v1).
- `eth_getTransactionReceipt`: retrieves confirmed receipts from `ReceiptStore`.

## Import Migration Rule
Use canonical `::traits::` paths for interface types; avoid non-canonical crate-root trait imports.
