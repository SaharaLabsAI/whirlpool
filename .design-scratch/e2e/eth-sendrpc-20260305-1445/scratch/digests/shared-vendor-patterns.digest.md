## Grounded facts
- Vendor reth examples use macro-based jsonrpsee namespace traits and `*ApiServer` impls (`vendor/reth/examples/node-custom-rpc/src/main.rs`, `vendor/reth/examples/rpc-db/src/myrpc_ext.rs`).
- Vendor reth pins `jsonrpsee = 0.26.0` (`vendor/reth/Cargo.toml`).

## [PROPOSED] deltas
- Mirror macro-based pattern with namespace `eth` for method set required by alloy client.
- Prefer `RpcResult<T>` signatures with alloy primitive-compatible params/results.
