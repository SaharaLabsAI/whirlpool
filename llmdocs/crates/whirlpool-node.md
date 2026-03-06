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
4. Build `WhirlpoolEvmConfig` and `EvmApplication<RethStateDb>`.
5. Provide `InMemoryTxPool` as tx source.
6. Wrap `FinalizationSink` and `EvmApplication` in `PersistingFinalizationSink` to enable block/receipt persistence.
7. Wrap app with `ApplicationAdapter`, construct `CommonwareEngine`, call `start()`.
8. Initialize `EthRpcContext` sharing the `RethStateDb` (as `BlockStorage`) and the `block_height` Arc with the `FinalizationSink`.
9. Start JSON-RPC server on `RPC_BIND_ADDR` (via `rpc_eth`).

## Key Types
- `PersistingFinalizationSink<DB, BS>`: `EventSink` implementation that persists finalized blocks to `BlockStorage` before delegating to the inner `FinalizationSink`.

## RPC
The RPC implementation lives in the separate `rpc-eth` crate. See `llmdocs/crates/rpc-eth.md`. It uses the `RethStateDb` as its `BlockStorage` backend.

## Import Migration Rule
Use canonical `::traits::` paths for interface types; avoid non-canonical crate-root trait imports.

## Integration Tests
Moved to `testing/integration-tests/` crate. See `llmdocs/crates/integration-tests.md`.
