# Task 12 Evidence: Basic Reth RPC Integration Tests

## Summary of changes

- Rewrote `testing/integration-tests/tests/rpc_evm_integration.rs` to use the public `rpc_eth::RpcConfig` and `rpc_eth::start_rpc_server()` API.
- Removed the legacy test harness that depended on `rpc_eth::context::EthRpcContext`, `rpc_eth::eth_handler::EthApiHandler`, and `rpc_eth::eth_api::EthApiServer`, which are now internal to `rpc-eth` under `#[cfg(test)]`.
- Added a lightweight `MockTxSource` plus a `start_test_rpc()` helper that boots a real RPC server against a fresh `state_reth` database in a temporary directory.
- Added the dependencies required by the new harness in `testing/integration-tests/Cargo.toml`: `app-evm` for `build_sahara_chain_spec()` / `SAHARA_CHAIN_ID`, and `reth-rpc-builder` for the server handle type.

## Tests added

### `tst4_server_returns_chain_id`
Verifies the server starts successfully and `eth_chainId` returns the configured Sahara chain ID (`313371`).

### `tst5_latest_block_number`
Verifies `eth_blockNumber` returns `0` when the backing database is empty.

### `tst6_balance_query_returns_zero_for_empty_db`
Verifies `eth_getBalance` returns `0` for `Address::ZERO` against an empty database.

### `tst7_eth_syncing_returns_false`
Verifies a raw JSON-RPC `eth_syncing` request returns `false`.

## Build output

Command:

```bash
nix develop --command bash -c "CARGO_BUILD_JOBS=1 RUSTFLAGS='-C codegen-units=1' cargo build -p integration-tests"
```

Result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.75s
```

## Test output

Command:

```bash
nix develop --command bash -c "CARGO_BUILD_JOBS=1 RUSTFLAGS='-C codegen-units=1' cargo test -p integration-tests"
```

Result:

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 2m 34s
Running tests/multinode_consensus.rs
 test test_four_node_consensus ... ok
Running tests/rpc_evm_integration.rs
 test tst6_balance_query_returns_zero_for_empty_db ... ok
 test tst7_eth_syncing_returns_false ... ok
 test tst4_server_returns_chain_id ... ok
 test tst5_latest_block_number ... ok

 test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```
