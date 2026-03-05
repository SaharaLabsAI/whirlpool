# Whirlpool Node Library: Config Constants

## Overview

The `whirlpool-node` crate library exports shared configuration constants used by the EVM node binary.

## Library Exports

`crates/whirlpool-node/src/lib.rs`: `pub mod config;`

### config.rs
- `pub const NAMESPACE: &[u8]` — `b"sahara-chain-v0"`
- `pub const BLOCK_INTERVAL: Duration` — 5 seconds
- `pub const VALIDATOR_SEED: u64` — 0
- `pub const BIND_ADDR: &str` — `"127.0.0.1:0"`
- `pub const RPC_BIND_ADDR: &str` — `"127.0.0.1:8545"`

## JSON-RPC Server Architecture

The `whirlpool-node` binary starts a `jsonrpsee` server for Ethereum compatibility.

### Component Map
- `EthRpcContext`: shared state across all RPC handlers.
  - `tx_pool`: `Arc<InMemoryTxPool>` for submitting raw transactions.
  - `state_db`: `Arc<RwLock<S>>` for account balance and nonce queries.
  - `receipt_store`: `Arc<ReceiptStore>` for retrieving confirmed transaction status.
  - `chain_id`: current Sahara chain ID (default 313371).
  - `block_height`: `Arc<AtomicU64>` reflecting the latest consensus height.
- `EthApiHandler`: implements the standard `eth_*` namespace.
- `ReceiptStore`: thread-safe `HashMap` mapping transaction hash to receipts.

### V1 Simplifications
- **Gas**: hardcoded 21,000 for transfers.
- **Gas Price**: hardcoded 1 gwei (1,000,000,000 wei).
- **Block ID**: supports "latest" and "pending" tags; specific block heights are not yet supported.
- **Receipts**: maintained in-memory; not persisted across node restarts.

## Dependency Graph

- **whirlpool-node** (lib) → used by `whirlpool-node` (bin) for config constants
- **whirlpool-node** (bin) → `jsonrpsee` + `alloy-rpc-types` for JSON-RPC implementation

## Binary: whirlpool-node (EVM)

Location: `crates/whirlpool-node/src/main.rs`
- Uses `EvmApplication` (from `app-evm`) with `InMemoryTxPool` (from `app`) and `TestStateDb` (local to main.rs)
- Implements `StateDb` and `revm::Database` traits for `TestStateDb`.
- Full EVM execution with state root progression
- Wiring: Starts consensus engine, then initializes and starts the JSON-RPC server.
- See `crates/whirlpool-node.md` for binary details

## Test Statistics

| Module | Test Count |
|--------|-----------|
| config.rs | 0 |
| main.rs (bin) | 0 |
| **Total (lib)** | **0** |


