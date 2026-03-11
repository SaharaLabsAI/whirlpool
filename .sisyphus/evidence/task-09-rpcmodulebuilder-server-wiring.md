# Task 09 Evidence: Rewire server.rs through RpcModuleBuilder

## Summary

Rewrote `crates/rpc-eth/src/server.rs` to wire WhirlpoolProvider, WhirlpoolTxPool, and WhirlpoolNetwork through reth's `RpcModuleBuilder` instead of legacy manual JSON-RPC setup.

## Changes

### `crates/rpc-eth/src/server.rs` (rewrite)
- Replaced 21-line legacy server with 55-line RpcModuleBuilder-based implementation
- New public API: `start_rpc_server(state_db, chain_spec, tx_source, addr) -> Result<(RpcServerHandle, SocketAddr), ...>`
- Uses `RpcModuleSelection::standard_modules()` for full Eth namespace
- Wires EthEvmConfig, NoopConsensus, TokioTaskExecutor
- Returns `RpcServerHandle` for lifecycle control and actual bound `SocketAddr`

### `crates/rpc-eth/src/provider.rs` (+16 lines)
- Added `PersistedBlockSubscriptions` trait implementation (required by `RpcModuleBuilder::build()`)
- Added `persisted_block_tx: watch::Sender<Option<BlockNumHash>>` field
- Constructor creates `watch::channel(None)` for the persisted block notification stream

### `crates/rpc-eth/Cargo.toml` (+3 lines)
- Added `reth-tokio-util` dependency (for `EventSender`)

### `crates/rpc-eth/tests/server_contract.rs` (new, 73 lines)
- `server_starts_and_returns_local_address`: verifies server binds to a port
- `server_responds_to_eth_chain_id`: verifies eth_chainId returns correct chain ID via HTTP JSON-RPC

## Blob Handling Approach

Blob (EIP-4844) exclusion is enforced at the **pool ingress layer** (Task 06, `pool.rs`):
- `WhirlpoolTxPool::add_external_transactions()` rejects any blob transaction with error "blob transactions (EIP-4844) are not supported"
- All `BlobStore` methods return empty results or `MissingSidecar` errors
- `pending_blob_fee` in `PoolStatus` is `None`

For `eth_blobBaseFee`: reth's default `EthApi` implementation reads `excess_blob_gas` from block headers. Since no blob transactions can enter the pool, and our chain doesn't produce blob-bearing blocks, the RPC returns the natural value from the chain state. No explicit override was needed.

## Build & Test Verification

```
cargo build -p rpc-eth  ✅ PASS
cargo test -p rpc-eth   ✅ 36/36 tests pass
```

### Test breakdown:
- 17 eth_handler (pre-existing)
- 5 convert_tests (pre-existing)
- 5 network_contract (pre-existing)
- 3 pool_contract (pre-existing)
- 4 provider_contract (pre-existing)
- 2 server_contract (NEW)
