# whirlpool-node: EVM Consensus Binary

## Summary
`whirlpool-node` runs Commonware consensus with `CompositeApplication` on Sahara Chain.

Location: `crates/whirlpool-node/`

## Dependency Boundaries
- `consensus`: core interface traits from `consensus::traits`.
- `consensus-simplex`: simplex adapter and engine.
- `app`: application adapter + `TxSource` trait.
- `app-composite`: consensus-facing mixed app implementation that drains the shared mempool and delegates by tx family.
- `app-evm`: pure EVM execution implementation + `app_evm::traits::StateProvider` + `build_sahara_chain_spec()` / `build_sahara_chain_spec_with_alloc()`.
- `tx-dispatch`: neutral mixed-tx classification used indirectly through `app-composite`.
- `reth-chainspec`: `ChainSpec` type for custom chain spec injection.
- `mempool`: `PersistentTxPool` for transaction storage.
- `state`: `StateDb` trait, `StateError`, and `BlockStorage` trait.
- `state-reth`: `RethStateDb` implementation for persistent state and block storage.
- `state-memory`: `InMemoryStateDb` implementation (test code only).
- `p2p-commonware`: network provider bridge.
- `rpc-eth`: `RpcConfig`, `start_rpc_server()` — Ethereum JSON-RPC server (reth-backed).
- `clap`: CLI argument parsing (derive macros).

## config.rs Configuration
1. **NodeArgs**: CLI parser supporting `--config <path>`, `--validator <hex>`, `--data-dir`, etc. (crates/whirlpool-node/src/config.rs:24)
2. **TomlConfig**: Serializable structure for persistent configuration (crates/whirlpool-node/src/config.rs:95).
3. **load_config(NodeArgs)**: Merges CLI overrides, TOML file settings, and hardcoded defaults (crates/whirlpool-node/src/config.rs:319).
4. **NodeConfig**: Root configuration validated for multi-validator deployments (crates/whirlpool-node/src/config.rs:55).

## node.rs Lifecycle
- **start_node(NodeConfig) -> NodeHandle**: Spawns consensus and RPC threads. Returns a handle with `Drop` implementation for teardown (crates/whirlpool-node/src/node.rs:50).
- **start_node_with_chain_spec(NodeConfig, Option<Arc<ChainSpec>>) -> NodeHandle**: Variant that accepts an optional custom `ChainSpec`. When provided, applies genesis allocations from the chain spec to the MDBX database via `RethStateDb::apply_genesis()` before starting the node. Used for integration tests with pre-funded accounts.
- **NodeHandle**: Tracks RPC/P2P addresses and the thread join handle (crates/whirlpool-node/src/node.rs:26).

### RPC Wiring (node.rs)
The node constructs an `rpc_eth::RpcConfig` with:
- `state_db`: `Arc<RethStateDb>` (shared with composite/EVM application state).
- `chain_spec`: `Arc<ChainSpec>` from `build_sahara_chain_spec()` (cloned before EVM config takes ownership).
- `tx_source`: `Arc<PersistentTxPool>` (implements `TxSource`).
- `addr`: RPC listen address from `NodeConfig`.

Then calls `rpc_eth::start_rpc_server(config)` to get `(RpcServerHandle, SocketAddr)`.

For mem RPC, the node shares the same `Arc<PersistentTxPool>` submit path and the finalized `Arc<InMemoryPersonalityStorage>` used by `PersistingFinalizationSink`, then starts `rpc_mem` with `TxSourceMemoryTxService::with_personality_storage(...)`. This keeps `mem_submitPersonality` unchanged while enabling finalized-only `mem_getPersonality` and `mem_getTransactionByHash` reads on the dedicated mem RPC server.

### Mixed App Wiring
- `PersistentTxPool` remains the single raw-byte mempool for both `eth_sendRawTransaction` and `mem_submitPersonality`.
- `CompositeApplication` drains that shared source during proposal.
- `CompositeApplication` classifies bytes into EVM vs mem lanes and delegates EVM execution to `app-evm`.
- `PersistingFinalizationSink` persists block receipts through `CompositeApplication::store_finalized_block()` and finalized personality writes by decoding mem transactions from the finalized block payload.

## main.rs Entrypoint
Minimal wrapper:
1. `NodeArgs::parse()`
2. `load_config(args)`
3. `start_node(config)`
4. `std::thread::park()` loop.

## Startup Recovery Flow
Recovery relies on two persistence layers working together:
1. **Application state** (MDBX): The node opens the persistent database and queries `CanonicalHeaders` for the highest stored block number. This `recovered_height` seeds the atomic `height` tracker and the engine's `height` config (crates/whirlpool-node/src/node.rs:89,109) so the application layer resumes at the correct block.
2. **Consensus journal** (Commonware Storage): The Simplex voter journals votes and certificates to `config.storage.runtime_dir()` (crates/whirlpool-node/src/node.rs:55). On restart the voter replays the journal to recover its view/round state. Without a persistent storage directory the journal is lost and consensus restarts from view 0.

## Key Types
- `PersistingFinalizationSink<DB, BS>`: `EventSink` implementation that persists finalized blocks to `BlockStorage` before delegating to the inner `FinalizationSink`.

## RPC
The node runs two RPC servers:
- `rpc-eth`: reth-backed `eth_*` methods served from `RethStateDb` (MDBX). Blob (EIP-4844) support is excluded.
- `rpc-mem`: `mem_submitPersonality` plus finalized-only `mem_getPersonality` and `mem_getTransactionByHash`, backed by the shared mempool submit path and finalized personality storage.

`mem_*` methods are not mounted on the Ethereum RPC listener; they are exposed only on `NodeHandle::mem_rpc_addr`.

## Import Migration Rule
Use canonical `::traits::` paths for interface types; avoid non-canonical crate-root trait imports.

## Integration Tests
Moved to `testing/integration-tests/` crate. See `agent-docs/crates/integration-tests.md`.
