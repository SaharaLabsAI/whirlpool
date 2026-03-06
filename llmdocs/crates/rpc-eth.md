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
- `eth_handler.rs`: `EthApiHandler<S: StateDb, B: BlockStorage>` implementing `EthApiServer`. Contains unit tests.
- `context.rs`: `EthRpcContext<S: StateDb, B: BlockStorage>` — shared context holding `Arc` handles to tx pool, state, block storage, receipt store, chain ID, and atomic block height.
- `receipt_store.rs`: `ReceiptStore` — thread-safe in-memory `HashMap<B256, TransactionReceipt>`.
- `server.rs`: `start_rpc_server()` — builds and starts the jsonrpsee server.

## Supported RPC Methods
- `eth_chainId`: returns the configured Sahara chain ID.
- `eth_gasPrice`: returns hardcoded 1 gwei (v1).
- `eth_getBalance`: returns balance from `StateDb` for a specified block.
- `eth_getTransactionCount`: returns nonce from `StateDb` for a specified block.
- `eth_sendRawTransaction`: pushes raw bytes directly into `InMemoryTxPool`.
- `eth_estimateGas`: returns hardcoded 21,000 gas (v1).
- `eth_getTransactionReceipt`: retrieves confirmed receipts from `ReceiptStore`.
- `eth_getBlockByNumber(number: BlockNumberOrTag, full_txs: bool)`: retrieves blocks from `BlockStorage`. Resolves `latest`/`finalized` to current node height.
- `eth_getBlockByHash(hash: B256, full_txs: bool)`: retrieves blocks from `BlockStorage`.

## Canonical Imports
- `rpc_eth::context::EthRpcContext`
- `rpc_eth::eth_api::EthApiServer`
- `rpc_eth::eth_handler::EthApiHandler`
- `rpc_eth::receipt_store::ReceiptStore`
- `rpc_eth::server::start_rpc_server`

## Key Design Notes
- `EthRpcContext` and `EthApiHandler` are generic over `S: StateDb` and `B: BlockStorage`.
- `validate_block_id()` supports `latest`, `finalized`, `earliest`, `pending`, and specific block numbers.
- `evm_block_to_rpc_block()` handles conversion from internal `EvmBlock` to `alloy_rpc_types::Block`.

## Status
Complete. Extracted from `whirlpool-node::rpc` module.
