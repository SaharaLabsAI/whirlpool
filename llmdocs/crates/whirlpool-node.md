# whirlpool-node: EVM Consensus Binary

## Summary
`whirlpool-node` runs Commonware consensus with `EvmApplication` on Sahara Chain.

Location: `crates/whirlpool-node/`

## Dependency Boundaries
- `consensus`: core interface traits from `consensus::traits`.
- `consensus-simplex`: simplex adapter and engine.
- `app`: application adapter + tx source implementations (`InMemoryTxPool`).
- `app-evm`: EVM app implementation + `app_evm::traits::StateProvider`.
- `state`: `StateDb` trait, `StateError`, and `BlockStorage` trait.
- `state-reth`: `RethStateDb` implementation for persistent state and block storage.
- `state-memory`: `InMemoryStateDb` implementation (test code only).
- `p2p-commonware`: network provider bridge.
- `rpc-eth`: Ethereum JSON-RPC server (extracted from former `rpc/` module).

## main.rs Wiring
1. Initialize Commonware runtime with **persistent storage directory** (`DEFAULT_RUNTIME_STORAGE_DIR`) so the consensus journal survives restarts.
2. Build Commonware network provider.
3. Open `RethStateDb` at `DEFAULT_DB_PATH` via `state_reth::open_state_db`.
4. Recover chain tip via `block_storage.get_latest_block_number()` and seed shared `height` Arc.
5. Build `WhirlpoolEvmConfig` and `EvmApplication<RethStateDb>`.
6. Provide `InMemoryTxPool` as tx source.
7. Wrap `FinalizationSink` and `EvmApplication` in `PersistingFinalizationSink` to enable block/receipt persistence.
8. Construct `CommonwareEngine` (passing recovered `height` Arc in `engine_config`), call `start()`.
9. Initialize `EthRpcContext` sharing the `RethStateDb` (as `BlockStorage`) and the `height` Arc (as `block_height`).
10. Start JSON-RPC server on `RPC_BIND_ADDR` (via `rpc_eth`).

## Startup Recovery Flow
Recovery relies on two persistence layers working together:
1. **Application state** (MDBX): The node opens the persistent database and queries `CanonicalHeaders` for the highest stored block number. This `recovered_height` seeds the atomic `height` tracker and the engine's `height` config (crates/whirlpool-node/src/main.rs:84,99) so the application layer (proposals, finalization sink, RPC) resumes at the correct block.
2. **Consensus journal** (Commonware Storage): The Simplex voter journals votes and certificates to `DEFAULT_RUNTIME_STORAGE_DIR` (crates/whirlpool-node/src/main.rs:32). On restart the voter replays the journal to recover its view/round state, preventing re-voting or re-proposing for already-decided views. Without a persistent storage directory the journal is lost and consensus restarts from view 0.

## Key Types
- `PersistingFinalizationSink<DB, BS>`: `EventSink` implementation that persists finalized blocks to `BlockStorage` before delegating to the inner `FinalizationSink`.

## RPC
The RPC implementation lives in the separate `rpc-eth` crate. See `llmdocs/crates/rpc-eth.md`. It uses the `RethStateDb` as its `BlockStorage` backend.

## Import Migration Rule
Use canonical `::traits::` paths for interface types; avoid non-canonical crate-root trait imports.

## Integration Tests
Moved to `testing/integration-tests/` crate. See `llmdocs/crates/integration-tests.md`.
