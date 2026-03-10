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

## main.rs Wiring
1. Parse CLI arguments via `clap::Parser` into `NodeArgs`, convert to `NodeConfig`.
2. Initialize Commonware runtime with persistent storage directory from `config.storage.runtime_dir()`.
3. Build Commonware network provider using `config.network.*` fields (namespace, listen/dialable addr, bootstrap peers, max message size).
4. Open `RethStateDb` at `config.storage.state_dir()` via `state_reth::open_state_db`.
5. Recover chain tip via `block_storage.get_latest_block_number()` and seed shared `height` Arc.
6. Build `WhirlpoolEvmConfig` and `EvmApplication<RethStateDb>`.
7. Open `PersistentTxPool` at `config.storage.mempool_dir()` as `TxSource`.
8. Wrap `FinalizationSink` and `EvmApplication` in `PersistingFinalizationSink`.
9. Construct `CommonwareEngine` (passing recovered `height` Arc in `engine_config`), call `start()`.
10. Initialize `EthRpcContext` sharing the `RethStateDb` and `height` Arc.
11. Start JSON-RPC server on `config.rpc.bind_addr`.

## Startup Recovery Flow
Recovery relies on two persistence layers working together:
1. **Application state** (MDBX): The node opens the persistent database and queries `CanonicalHeaders` for the highest stored block number. This `recovered_height` seeds the atomic `height` tracker and the engine's `height` config (crates/whirlpool-node/src/main.rs:84,99) so the application layer (proposals, finalization sink, RPC) resumes at the correct block.
2. **Consensus journal** (Commonware Storage): The Simplex voter journals votes and certificates to `config.storage.runtime_dir()` (crates/whirlpool-node/src/main.rs:32). On restart the voter replays the journal to recover its view/round state, preventing re-voting or re-proposing for already-decided views. Without a persistent storage directory the journal is lost and consensus restarts from view 0.

## Key Types
- `PersistingFinalizationSink<DB, BS>`: `EventSink` implementation that persists finalized blocks to `BlockStorage` before delegating to the inner `FinalizationSink`.

## RPC
The RPC implementation lives in the separate `rpc-eth` crate. See `llmdocs/crates/rpc-eth.md`. It uses the `RethStateDb` as its `BlockStorage` backend.

## Import Migration Rule
Use canonical `::traits::` paths for interface types; avoid non-canonical crate-root trait imports.

## Integration Tests
Moved to `testing/integration-tests/` crate. See `llmdocs/crates/integration-tests.md`.
