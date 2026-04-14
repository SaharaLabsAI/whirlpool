# Whirlpool Node Library: Config Constants

## Overview

The `whirlpool-node` crate library exports a comprehensive configuration system including constants, CLI argument parsing via `clap`, and a nested configuration hierarchy used by the EVM node binary.

## Library Exports

`crates/node/src/lib.rs`: `pub mod config;`

### config.rs
- Types: `NodeArgs` (clap), `TomlConfig` (serde), `NodeConfig` (validated), `NetworkConfig`, `IdentityConfig`, `RpcConfig`, `StorageConfig`, `ConsensusStartupConfig`
- Functions: `load_config(args: NodeArgs) -> Result<NodeConfig, ConfigError>` (crates/node/src/config.rs:319)
- Impl: `From<NodeArgs> for NodeConfig`, `Default` for all config types
- Constants (defaults): `APPLICATION_NAMESPACE`, `NAMESPACE`, `BLOCK_INTERVAL`, `VALIDATOR_SEED`, `BIND_ADDR`, `RPC_BIND_ADDR`, `DEFAULT_DATA_DIR`, `DEFAULT_MAX_MESSAGE_SIZE`
- Tests: file-separated in `crates/node/src/tests/config.rs` via `#[path = "tests/config.rs"] mod tests;`

### node.rs
- `start_node(config: NodeConfig) -> Result<NodeHandle, ...>`: Spawns the node lifecycle thread (crates/node/src/node.rs:50).
- `NodeHandle`: Provides `rpc_addr`, `p2p_addr`, and handles graceful thread park/unpark on `Drop` (crates/node/src/node.rs:26).

## JSON-RPC Server Architecture

The JSON-RPC implementation has been extracted to the `rpc-eth` crate. See `agent-docs/crates/rpc-eth.md` for full details.

The `whirlpool-node` binary wires `rpc-eth` via `rpc_eth::context::EthRpcContext` and `rpc_eth::server::start_rpc_server` (crates/node/src/node.rs:140).

## Dependency Graph

- **whirlpool-node** (lib) → exported `config` and `node` modules
- **whirlpool-node** (bin) → `whirlpool-node` (lib) + `rpc-eth`

## Binary: whirlpool-node (EVM)

Location: `crates/node/src/main.rs`
- Configuration: Parsed via `NodeArgs`, loaded via `load_config` supporting CLI + TOML.
- Execution: Delegates to `start_node(config)` and parks the main thread.
- Persistent state: MDBX state stored at `config.storage.state_dir()`.
- See `crates/whirlpool-node.md` for binary details

## Test Statistics

| Module | Test Count |
|--------|-----------|
| config.rs | 8 |
| main.rs (bin) | 1 |
| **Total** | **9** |
