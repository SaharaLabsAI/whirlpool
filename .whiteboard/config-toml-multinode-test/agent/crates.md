# Crate Index

## Affected Crates

### whirlpool-node (MODIFY — primary)
- Add `toml`, `serde` (Deserialize) deps
- config.rs: Add `TomlConfig` struct, `--config`/`--validator` CLI args, TOML loading + merge logic
- main.rs: Replace hardcoded validators with config-driven list, extract `start_node()` for testability
- lib.rs: Re-export `start_node()` and config types for integration test use

### integration-tests (MODIFY — test harness)
- Cargo.toml: Add `whirlpool-node` dependency
- New test file: `tests/multinode_consensus.rs` — 4-node P2P + block height test

## Unaffected Crates
- consensus, consensus-simplex, p2p, p2p-commonware, app, app-evm, state, state-memory, state-reth, mempool, mempool-mdbx, rpc-eth — no changes needed. The multi-validator support is purely a configuration concern at the node binary level.

## New Crates
- None

## New Dependencies
- `toml` (crate) — TOML deserialization
- `serde` (crate, Deserialize feature) — struct deserialization
