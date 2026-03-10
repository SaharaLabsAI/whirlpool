# Whirlpool Node Library: Config Constants

## Overview

The `whirlpool-node` crate library exports a comprehensive configuration system including constants, CLI argument parsing via `clap`, and a nested configuration hierarchy used by the EVM node binary.

## Library Exports

`crates/whirlpool-node/src/lib.rs`: `pub mod config;`

### config.rs
- Types: `NodeArgs` (clap derive, 12 CLI fields), `NodeConfig` (top-level config), `NetworkConfig`, `IdentityConfig`, `RpcConfig`, `StorageConfig`, `ConsensusStartupConfig`
- Functions: `parse_bootstrap_peer(s: &str) -> Result<BootstrapPeer, String>`
- Impl: `From<NodeArgs> for NodeConfig`
- Constants (defaults): `APPLICATION_NAMESPACE`, `NAMESPACE`, `BLOCK_INTERVAL`, `VALIDATOR_SEED`, `BIND_ADDR`, `RPC_BIND_ADDR`, `DEFAULT_DATA_DIR`, `DEFAULT_MAX_MESSAGE_SIZE`

## JSON-RPC Server Architecture

The JSON-RPC implementation has been extracted to the `rpc-eth` crate. See `agent-docs/crates/rpc-eth.md` for full details.

The `whirlpool-node` binary wires `rpc-eth` via `rpc_eth::context::EthRpcContext` and `rpc_eth::server::start_rpc_server`.

## Dependency Graph

- **whirlpool-node** (lib) → used by `whirlpool-node` (bin) for config constants
- **whirlpool-node** (bin) → `rpc-eth` for JSON-RPC server

## Binary: whirlpool-node (EVM)

Location: `crates/whirlpool-node/src/main.rs`
- Configuration: Driven by CLI flags parsed via `clap` into `NodeConfig` hierarchy.
- Persistent state: Opens or creates MDBX environment at `config.storage.state_dir()`.
- Implements `StateDb`, `StateProvider`, and `revm::Database` traits via `RethStateDb`.
- Full EVM execution with persistent state root progression.
- Wiring: Starts consensus engine, then initializes and starts the JSON-RPC server.
- See `crates/whirlpool-node.md` for binary details

## Test Statistics

| Module | Test Count |
|--------|-----------|
| config.rs | 8 |
| main.rs (bin) | 1 |
| **Total** | **9** |


