# rpc-eth: Ethereum JSON-RPC Server

## Summary
`rpc-eth` implements a minimal Ethereum JSON-RPC server for the Whirlpool node, supporting basic `eth_*` methods needed for ETH balance transfers via alloy clients.

Location: `crates/rpc-eth/`

## Dependency Boundaries
- `app`: `InMemoryTxPool` and `TxSource` trait (`app::tx_source`).
- `state`: `StateDb` trait (`state::traits`).
- `jsonrpsee`: JSON-RPC server framework (0.26.0) — server and macros features.
- `alloy-primitives`: Ethereum primitives (Address, B256, U256, Bytes).
- `alloy-rpc-types`: Ethereum RPC types (BlockId, TransactionReceipt, TransactionRequest).
- `async-trait`: async trait support for `EthApiServer` impl.
- `tracing`: structured logging.

## Module Layout
- `eth_api.rs`: `EthApi` trait — jsonrpsee `#[rpc(server, namespace = "eth")]` macro defining 7 RPC methods.
- `eth_handler.rs`: `EthApiHandler<S: StateDb>` implementing `EthApiServer`. Contains unit tests.
- `context.rs`: `EthRpcContext<S: StateDb>` — shared context holding `Arc` handles to tx pool, state, receipts, chain ID, block height.
- `receipt_store.rs`: `ReceiptStore` — thread-safe in-memory `HashMap<B256, TransactionReceipt>`.
- `server.rs`: `start_rpc_server()` — builds and starts the jsonrpsee server.

## Supported RPC Methods
- `eth_chainId`: returns the configured Sahara chain ID.
- `eth_gasPrice`: returns hardcoded 1 gwei (v1).
- `eth_getBalance`: returns balance from `StateDb` for "latest" block. Handles `Result` and returns `RpcResult` errors.
- `eth_getTransactionCount`: returns nonce from `StateDb` for "latest" block. Handles `Result` and returns `RpcResult` errors.
- `eth_sendRawTransaction`: pushes raw bytes directly into `InMemoryTxPool`.
- `eth_estimateGas`: returns hardcoded 21,000 gas (v1).
- `eth_getTransactionReceipt`: retrieves confirmed receipts from `ReceiptStore`.

## Canonical Imports
- `rpc_eth::context::EthRpcContext`
- `rpc_eth::eth_api::EthApiServer`
- `rpc_eth::eth_handler::EthApiHandler`
- `rpc_eth::receipt_store::ReceiptStore`
- `rpc_eth::server::start_rpc_server`

## Key Design Notes
- `EthRpcContext` is currently concrete over `Arc<InMemoryTxPool>` (not generic over `TxSource`).
- `validate_block_id()` rejects all block IDs except `latest` and `pending`.
- Hardcoded gas values (`TRANSFER_GAS=21000`, `GAS_PRICE_WEI=1gwei`) are v1 placeholders.

## Status
Complete. Extracted from `whirlpool-node::rpc` module.
