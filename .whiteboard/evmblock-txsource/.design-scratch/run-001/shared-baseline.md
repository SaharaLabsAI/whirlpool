# Shared Baseline — evmblock-txsource

## Prior Design

This is Sub-Intent 2 of the evm-tx-execution design (`docs/design/evm-tx-execution/`).
Sub-Intent 1 implemented the EVM execution engine with propose/verify paths.
TxSource was explicitly deferred as out-of-scope.

## Current State

- `TxSource` trait: `crates/app/src/traits.rs:23-25` — `fn pending(&self) -> Vec<Vec<u8>>`
- `NoopTxSource`: returns empty vec, used in whirlpool-node main.rs
- `EvmApplication.tx_source`: `Arc<dyn TxSource + Send + Sync>` in `crates/app-evm/src/executor.rs`
- `propose()` calls `tx_source.pending()` to get raw EIP-2718 encoded transactions
- Node wiring: `crates/whirlpool-node/src/main.rs:130` — `Arc::new(NoopTxSource)`

## Design Scope

Implement a minimal in-memory mempool that:
1. Stores pending transactions as `Vec<u8>` (EIP-2718 encoded)
2. Provides a `push(tx: Vec<u8>)` method to add transactions
3. Implements `TxSource::pending()` to return and drain stored transactions
4. Is thread-safe (`Send + Sync`)
5. Wires into whirlpool-node replacing NoopTxSource

## Out of Scope

- JSON-RPC endpoint for tx submission (Sub-Intent 3)
- Transaction validation before pool insertion
- Transaction ordering/priority (gas price sorting)
- Pool size limits / eviction policies
- Duplicate detection
- Nonce management
