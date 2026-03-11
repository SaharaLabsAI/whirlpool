# Grounding Map

## Existing rpc-eth Surface
- Current files: `crates/rpc-eth/src/context.rs`, `crates/rpc-eth/src/eth_api.rs`, `crates/rpc-eth/src/eth_handler.rs`, `crates/rpc-eth/src/receipt_store.rs`, `crates/rpc-eth/src/server.rs`, `crates/rpc-eth/src/lib.rs`.
- Planned replacements/additions: `crates/rpc-eth/src/provider.rs`, `crates/rpc-eth/src/pool.rs`, `crates/rpc-eth/src/network.rs`, `crates/rpc-eth/src/convert.rs`, rewritten `crates/rpc-eth/src/server.rs`, rewritten `crates/rpc-eth/src/lib.rs`.
- Removal seams: `EthRpcContext`, `EthApiHandler`, `EthApiServer`, and `receipt_store.rs` become legacy surfaces to delete or stop exporting once `RpcModuleBuilder` is live.

## whirlpool-node Touchpoints
- Candidate wiring files: `crates/whirlpool-node/src/main.rs`, `crates/whirlpool-node/src/node.rs`.
- Integration expectation: node startup continues unchanged except for RPC server construction and handle management.

## Integration Test Harness
- Existing harness file: `testing/integration-tests/tests/rpc_integration.rs`.
- Existing behavior: starts a JSON-RPC server in-process and drives it with alloy `ProviderBuilder`.
- Planned reuse: extend this file to match reth `rpc-builder` HTTP setup patterns instead of creating a new harness.

## Vendor Pattern References
- `vendor/reth/crates/rpc/rpc-builder/tests/it/http.rs` is the canonical reference for HTTP startup, typed clients, and parameter permutations.
- `vendor/reth/crates/rpc/rpc-eth-api/src/core.rs` and `vendor/reth/crates/storage/provider` define the trait signatures the provider adapter must satisfy.
- Vendor files are read-only and serve only as signature/pattern references.

## Build Order Notes
- `Cargo.toml` dependency updates must land before any new adapter module compiles.
- `provider.rs` must exist before `server.rs` can instantiate `RpcModuleBuilder`.
- `convert.rs`, `pool.rs`, and `network.rs` must be in place before the new server wiring compiles.
- `lib.rs` cleanup and `whirlpool-node` wiring must follow the server rewrite to avoid broken imports.
