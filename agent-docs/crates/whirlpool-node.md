# whirlpool-node: EVM Consensus Binary

## Summary
`whirlpool-node` runs Commonware consensus with the pure EVM app and a dedicated chainspec crate boundary.

## Location
`crates/node/`

## Dependency Boundaries
- `chainspec`: Sahara semantic genesis config/builders + chain-id + validator-registry reader seam.
- `app-evm-execution`: EVM runtime/config/execution (`WhirlpoolEvmConfig`, `EvmApplication`).
- `consensus` / `consensus-simplex`: consensus traits + adapter/engine.
- `app-traits` / `app-primitives`: app adapter traits and concrete block primitives.
- `state` / `app-evm-state` / `mempool-mdbx`: persistence and tx source.
- `network-commonware`: networking.
- `rpc-eth`: Ethereum JSON-RPC server.

## node.rs Lifecycle
- `start_node(NodeConfig) -> NodeHandle`
- `start_node_with_chain_spec(NodeConfig, Option<Arc<ChainSpec>>) -> NodeHandle`
- Startup validates supplied genesis alloc against Sahara hard cap via `chainspec::native_token::validate_genesis_alloc`.

## Runtime Wiring
- Default chainspec construction uses `chainspec::genesis::build_sahara_chain_spec_from(SaharaGenesisConfig { ... })`.
- Simplex membership is decoded via `chainspec::validators::try_simplex_validators_from_chain_spec(...)`.
- Node startup still fails early when the local signer is not present in the resolved simplex validator set.
- Optional genesis bootstrap mode:
  - `--genesis-bootstrap-dkg`
  - `--genesis-bootstrap-validator-count`
  - `--genesis-dkg-session-dir`
  - `--genesis-dkg-dealer-pubkey`
- `run_genesis_bootstrap()` invokes `consensus-manager` trusted-dealer generation and exits without starting the node.
- Normal startup optionally loads local bundle material from `genesis_dkg_session_dir`; on load/validation failure startup hard-fails before engine start.
- Startup rejects `--genesis-dkg-dealer-pubkey` when no `--genesis-dkg-session-dir` is configured.
- When BLS material is loaded, node derives `FullDkgOutputV1` (`dealers`, `players`, encoded `public_polynomial`) and wires it into `WhirlpoolEvmConfig` via `with_current_full_dkg_output(...)`.
- Header `extra_data` is strict canonical from genesis; there is no CLI/TOML migration-height gate.
- `EvmApplication` is wired through `ApplicationAdapter` + `PersistingFinalizationSink`.
- Node no longer routes any epoch-boundary hook selection through chainspec; the boundary path is the single built-in `app-evm-execution`/`evm-precompiles` integration.
- Config parsing helpers in `src/config/parse.rs` use plain `pub` visibility inside the private `config::parse` module (no scoped `pub(super)` / `pub(crate)` modifiers), with parent-module imports controlling the crate-visible surface.

## RPC
- `rpc-eth` is the only server wired by `whirlpool-node`.
