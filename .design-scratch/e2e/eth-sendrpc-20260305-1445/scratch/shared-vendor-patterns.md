# Shared Vendor Patterns

## Grounded facts
- Vendor reth custom RPC example uses jsonrpsee proc macro trait declarations with namespace attributes:
  - `#[rpc(server, namespace = "txpoolExt")]`
  - trait methods annotated by `#[method(name = "...")]`
  - implementation via generated `*ApiServer` trait.
  Evidence: `vendor/reth/examples/node-custom-rpc/src/main.rs`.
- Vendor rpc-db example similarly uses:
  - `#[rpc(server, namespace = "myrpcExt")]`
  - return type `EthResult<Option<Block>>`.
  Evidence: `vendor/reth/examples/rpc-db/src/myrpc_ext.rs`.
- Vendor workspace pins `jsonrpsee = "0.26.0"`.
  Evidence: `vendor/reth/Cargo.toml`.

## jsonrpsee patterns relevant to this design
- Namespace is set at trait level; method names become `eth_<method>` by namespace.
- Server lifecycle commonly uses `ServerBuilder::default().build(addr).await` then `start(module)`.
- Methods return `RpcResult<T>` with typed params/returns compatible with serde.

## [PROPOSED] deltas
- Use the same macro-based server trait pattern in `whirlpool-node` for alignment and reduced boilerplate.
- Keep API surface minimal to required seven ETH methods.
- Prefer returning alloy primitive types (`U64/U256/B256/Bytes`) and ETH receipt types that alloy client expects.

## Vendor runtime constraints
- Vendor crates are read-only references; no modifications under `vendor/`.
- Local workspace should mirror API conventions, not vendor internal module architecture.
