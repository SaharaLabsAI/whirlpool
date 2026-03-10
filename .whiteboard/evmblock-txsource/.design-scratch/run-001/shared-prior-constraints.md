# Shared Prior Constraints — evmblock-txsource

## From evm-tx-execution design (Sub-Intent 1)

1. `TxSource::pending()` returns `Vec<Vec<u8>>` — EIP-2718 encoded raw transactions
2. `EvmApplication` takes `Arc<dyn TxSource + Send + Sync>` — must be thread-safe
3. `propose()` calls `pending()` then `decode_transactions()` which filter_maps invalid txs
4. `verify()` does NOT use TxSource — uses `block.transactions` directly
5. No validation contract on TxSource — it's the executor's job to handle bad txs
6. All existing tests use `MockTxSource` or `NoopTxSource` — real impl adds no test regression risk

## Constraints on new implementation

- MUST implement `TxSource` trait exactly as defined
- MUST be `Send + Sync` for `Arc<dyn TxSource + Send + Sync>`
- MUST NOT add new dependencies to `app` crate (std::sync suffices)
- MUST NOT change the `TxSource` trait signature
- SHOULD keep `NoopTxSource` for test usage
