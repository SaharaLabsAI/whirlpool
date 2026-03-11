# whirlpool-node: EVM Consensus Binary

## Summary
`whirlpool-node` runs Commonware consensus with `EvmApplication` on Sahara Chain.

Location: `crates/whirlpool-node/`

## Dependency Boundaries
- `consensus`: core interface traits from `consensus::traits`.
- `consensus-simplex`: simplex adapter and engine.
- `app`: application adapter + `TxSource` trait.
- `app-evm`: EVM app implementation + `app_evm::traits::StateProvider`.
- `mempool`: `PersistentTxPool` for transaction storage.
- `state`: `StateDb` trait, `StateError`, and `BlockStorage` trait.
- `state-reth`: `RethStateDb` implementation for persistent state and block storage.
- `state-memory`: `InMemoryStateDb` implementation (test code only).
- `p2p-commonware`: network provider bridge.
- `rpc-eth`: Ethereum JSON-RPC server (extracted from former `rpc/` module).
- `clap`: CLI argument parsing (derive macros).

## config.rs Configuration
1. **NodeArgs**: CLI parser supporting `--config <path>`, `--validator <hex>`, `--data-dir`, etc. (crates/whirlpool-node/src/config.rs:24)
2. **TomlConfig**: Serializable structure for persistent configuration (crates/whirlpool-node/src/config.rs:95).
3. **load_config(NodeArgs)**: Merges CLI overrides, TOML file settings, and hardcoded defaults (crates/whirlpool-node/src/config.rs:319).
4. **NodeConfig**: Root configuration validated for multi-validator deployments (crates/whirlpool-node/src/config.rs:55).

## node.rs Lifecycle
- **start_node(NodeConfig) -> NodeHandle**: Spawns consensus and RPC threads. Returns a handle with `Drop` implementation for teardown (crates/whirlpool-node/src/node.rs:50).
- **NodeHandle**: Tracks RPC/P2P addresses and the thread join handle (crates/whirlpool-node/src/node.rs:26).

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
The RPC implementation lives in the separate `rpc-eth` crate. See `agent-docs/crates/rpc-eth.md`. It uses the `RethStateDb` as its `BlockStorage` backend.

## Import Migration Rule
Use canonical `::traits::` paths for interface types; avoid non-canonical crate-root trait imports.

## Integration Tests
Moved to `testing/integration-tests/` crate. See `agent-docs/crates/integration-tests.md`.
