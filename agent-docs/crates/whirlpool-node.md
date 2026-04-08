# whirlpool-node: EVM Consensus Binary

## Summary
`whirlpool-node` runs Commonware consensus with the pure EVM app and a dedicated chainspec crate boundary.

## Location
`crates/node/`

## Dependency Boundaries
- `chainspec`: Sahara chain-spec builders + chain-id + validator-registry reader seam.
- `app-evm`: EVM runtime/config/execution (`WhirlpoolEvmConfig`, `EvmApplication`).
- `consensus` / `consensus-simplex`: consensus traits + adapter/engine.
- `state` / `state-reth` / `mempool-mdbx`: persistence and tx source.
- `p2p-commonware`: networking.
- `rpc-eth`: Ethereum JSON-RPC server.

## node.rs Lifecycle
- `start_node(NodeConfig) -> NodeHandle`
- `start_node_with_chain_spec(NodeConfig, Option<Arc<ChainSpec>>) -> NodeHandle`
- Startup validates supplied genesis alloc against native-token cap.

## Runtime Wiring
- Default chainspec construction uses `chainspec::build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(...)`.
- Simplex membership is decoded via `chainspec::try_simplex_validators_from_chain_spec(...)`.
- Node startup still fails early when the local signer is not present in the resolved simplex validator set.
- `EvmApplication` is wired through `ApplicationAdapter` + `PersistingFinalizationSink`.

## RPC
- `rpc-eth` is the only server wired by `whirlpool-node`.
