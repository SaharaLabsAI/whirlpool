# whirlpool-node: EVM Consensus Binary

## Summary
`whirlpool-node` runs Commonware consensus with `EvmApplication` on Sahara Chain.

Location: `crates/whirlpool-node/`

## Dependency Boundaries
- `consensus`: core interface traits from `consensus::traits`.
- `consensus-simplex`: simplex adapter and engine.
- `app`: application adapter + tx source implementations.
- `app-evm`: EVM app implementation + `app_evm::traits::StateProvider`.
- `state`: `InMemoryStateDb` implementation for state storage.
- `p2p-commonware`: network provider bridge.

## Canonical Trait Imports Used by Node
- `consensus::traits::ConsensusEngine`
- `app_evm::traits::StateProvider`

## main.rs Wiring
1. Initialize runtime and `FinalizationSink`.
2. Build Commonware network provider.
3. Construct `TestStateDb(InMemoryStateDb)` implementing `StateProvider` and `revm::Database`.
4. Build `WhirlpoolEvmConfig` and `EvmApplication`.
5. Provide `InMemoryTxPool` as tx source.
6. Wrap app with `ApplicationAdapter`, construct `CommonwareEngine`, call `start()`.

## Import Migration Rule
Use canonical `::traits::` paths for interface types; avoid non-canonical crate-root trait imports.
