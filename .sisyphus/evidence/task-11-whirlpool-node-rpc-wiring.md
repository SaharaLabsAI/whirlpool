# Task 11 Evidence: Update whirlpool-node for RpcConfig startup

## Summary

Updated `crates/whirlpool-node/src/node.rs` to use the new `rpc_eth::RpcConfig` public API instead of the removed legacy `rpc::context::EthRpcContext` + `rpc::server::start_rpc_server(ctx, addr)` pattern.

## Changes

### `crates/whirlpool-node/src/node.rs`
- Removed `SAHARA_CHAIN_ID` import (no longer needed — chain ID comes from chain_spec)
- Changed `WhirlpoolEvmConfig::new(chain_spec)` to `WhirlpoolEvmConfig::new(chain_spec.clone())` to preserve `chain_spec` for RPC config
- Replaced legacy `EthRpcContext::new(tx_pool, state_db, block_storage, SAHARA_CHAIN_ID)` + `rpc::server::start_rpc_server(rpc_ctx, addr)` with:
  ```rust
  let rpc_config = rpc::RpcConfig {
      state_db: block_storage.clone(),
      chain_spec,
      tx_source: tx_pool.clone(),
      addr: config.rpc.bind_addr,
  };
  let (_rpc_handle, rpc_addr) = rpc::start_rpc_server(rpc_config).await...
  ```

## Build & Test Verification

```
cargo build -p whirlpool-node  ✅ PASS
cargo test -p rpc-eth          ✅ 36/36 tests pass
```
