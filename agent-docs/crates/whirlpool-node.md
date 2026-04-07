# whirlpool-node: EVM Consensus Binary

## Summary
`whirlpool-node` runs Commonware consensus with the pure `app-evm` application on Sahara Chain.

## Location
`crates/whirlpool-node/`

## Dependency Boundaries
- `consensus`: core interface traits from `consensus::traits`.
- `consensus-simplex`: simplex adapter and engine.
- `app`: application adapter + `TxSource` trait.
- `app-evm`: pure EVM execution implementation plus Sahara chain-spec builders.
- `native-token`: shared hard-cap validation for genesis allocs.
- `mempool`: `PersistentTxPool` for transaction storage.
- `state`: `StateDb` trait and `BlockStorage` trait.
- `state-reth`: `RethStateDb` implementation for persistent state and block storage.
- `state-memory`: `InMemoryStateDb` implementation for tests only.
- `p2p-commonware`: network provider bridge.
- `rpc-eth`: Ethereum JSON-RPC server (reth-backed).

## node.rs Lifecycle
- `start_node(NodeConfig) -> NodeHandle`: spawns consensus and the Ethereum RPC thread set.
- `start_node_with_chain_spec(NodeConfig, Option<Arc<ChainSpec>>) -> NodeHandle`: optional custom genesis variant for integration tests.
- Startup validates any supplied `ChainSpec.genesis.alloc` against the shared native-token cap before the worker thread starts.
- `NodeHandle`: tracks RPC/P2P addresses, validator public key, and the worker thread handle.

## Runtime Wiring
- `PersistentTxPool` remains the transaction source for the EVM app and Ethereum RPC submit path.
- `EvmApplication` is wired directly into `ApplicationAdapter` and `PersistingFinalizationSink`.
- `PersistingFinalizationSink` persists finalized blocks only; personality/mem persistence was removed from the node path.
- `rpc_eth::start_rpc_server()` is the only RPC server startup in `node.rs`.

## RPC
The node runs one RPC server:
- `rpc-eth`: reth-backed `eth_*` methods served from `RethStateDb` (MDBX). Blob (EIP-4844) support is excluded.

`rpc-mem` remains a separate crate under `crates/mem/` but is no longer wired by `whirlpool-node`.

## Key Types
- `PersistingFinalizationSink<DB, BS>`: `EventSink` implementation that persists finalized blocks to `BlockStorage` before delegating to the inner `FinalizationSink`.

## Integration Tests
Moved to `testing/integration-tests/` crate. See `agent-docs/crates/integration-tests.md`.
