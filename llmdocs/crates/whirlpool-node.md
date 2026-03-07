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
1. Initialize runtime and `FinalizationSink`.
2. Build Commonware network provider.
3. Open `RethStateDb` at `DEFAULT_DB_PATH` via `state_reth::open_state_db`.
4. Recover chain tip via `block_storage.get_latest_block_number()` and seed shared `height` Arc.
5. Build `WhirlpoolEvmConfig` and `EvmApplication<RethStateDb>`.
6. Provide `InMemoryTxPool` as tx source.
7. Wrap `FinalizationSink` and `EvmApplication` in `PersistingFinalizationSink` to enable block/receipt persistence.
8. Wrap app with `ApplicationAdapter`, construct `CommonwareEngine` (passing recovered `initial_height`), call `start()`.
9. Initialize `EthRpcContext` sharing the `RethStateDb` (as `BlockStorage`) and the `block_height` Arc with the `FinalizationSink`.
10. Start JSON-RPC server on `RPC_BIND_ADDR` (via `rpc_eth`).

## Startup Recovery Flow
On restart, the node opens the persistent MDBX database before consensus starts. It performs O(log N) lookup in the `CanonicalHeaders` table to find the highest stored block number. This `recovered_height` is used to:
- Seed the atomic `height` tracker shared by the app, sink, and RPC.
- Configure the Simplex engine's `initial_height` so consensus resumes correctly from the last finalized block.
- Ensure the JSON-RPC server reports the correct block tip immediately.

## Key Types
- `PersistingFinalizationSink<DB, BS>`: `EventSink` implementation that persists finalized blocks to `BlockStorage` before delegating to the inner `FinalizationSink`.

## RPC
The RPC implementation lives in the separate `rpc-eth` crate. See `llmdocs/crates/rpc-eth.md`. It uses the `RethStateDb` as its `BlockStorage` backend.

## Import Migration Rule
Use canonical `::traits::` paths for interface types; avoid non-canonical crate-root trait imports.

## Integration Tests
Moved to `testing/integration-tests/` crate. See `llmdocs/crates/integration-tests.md`.
