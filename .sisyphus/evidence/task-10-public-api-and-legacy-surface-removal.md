# Task 10 Evidence: Public API and Legacy Surface Removal

## Summary of changes

- Rewrote `crates/rpc-eth/src/lib.rs` to expose a new top-level public API:
  - `RpcConfig`
  - `RpcError`
  - `start_rpc_server(config: RpcConfig)`
- Internalized adapter implementation modules in `lib.rs` using private `*_impl` modules:
  - `convert_impl`, `network_impl`, `pool_impl`, `provider_impl`, `server_impl`
- Kept legacy modules private and test-only to preserve inline unit tests in `eth_handler.rs`:
  - `#[cfg(test)] mod context;`
  - `#[cfg(test)] mod eth_api;`
  - `#[cfg(test)] mod eth_handler;`
  - `#[cfg(test)] mod receipt_store;`
- Added `thiserror = "2"` to `crates/rpc-eth/Cargo.toml` for `RpcError` derivation.

## Removed/internalized legacy public surface

- Removed direct `pub mod` exposure of:
  - `context`
  - `eth_api`
  - `eth_handler`
  - `receipt_store`
- Adapter module implementation moved behind private internals (`*_impl`) in `lib.rs`.

## New public API surface

- `RpcConfig`
- `RpcError`
- `start_rpc_server(config: RpcConfig) -> Result<(RpcServerHandle, SocketAddr), RpcError>`

## Build/test verification

### Build command

```bash
nix develop --command bash -c "CARGO_BUILD_JOBS=1 RUSTFLAGS='-C codegen-units=1' cargo build -p rpc-eth"
```

Result: **PASS**

### Test command

```bash
nix develop --command bash -c "CARGO_BUILD_JOBS=1 RUSTFLAGS='-C codegen-units=1' cargo test -p rpc-eth"
```

Result: **PASS**

Test totals observed:
- `src/lib.rs` unit tests: 17 passed (`eth_handler`)
- `tests/convert_tests.rs`: 5 passed
- `tests/network_contract.rs`: 5 passed
- `tests/pool_contract.rs`: 3 passed
- `tests/provider_contract.rs`: 4 passed
- `tests/server_contract.rs`: 2 passed

Total: **36 passed, 0 failed**
