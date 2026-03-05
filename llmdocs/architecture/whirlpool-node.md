# Whirlpool Node Library: Config Constants

## Overview

The `whirlpool-node` crate library exports shared configuration constants used by the EVM node binary.

## Library Exports

`crates/whirlpool-node/src/lib.rs`: `pub mod config;`

### config.rs
- `pub const NAMESPACE: &[u8]` — `b"sahara-chain-v0"`
- `pub const BLOCK_INTERVAL: Duration` — 5 seconds
- `pub const VALIDATOR_SEED: u64` — 0
- `pub const BIND_ADDR: &str` — `"127.0.0.1:0"`
- `pub const RPC_BIND_ADDR: &str` — `"127.0.0.1:8545"`

## JSON-RPC Server Architecture

The JSON-RPC implementation has been extracted to the `rpc-eth` crate. See `llmdocs/crates/rpc-eth.md` for full details.

The `whirlpool-node` binary wires `rpc-eth` via `rpc_eth::context::EthRpcContext` and `rpc_eth::server::start_rpc_server`.

## Dependency Graph

- **whirlpool-node** (lib) → used by `whirlpool-node` (bin) for config constants
- **whirlpool-node** (bin) → `rpc-eth` for JSON-RPC server

## Binary: whirlpool-node (EVM)

Location: `crates/whirlpool-node/src/main.rs`
- Uses `EvmApplication` (from `app-evm`) with `InMemoryTxPool` (from `app`) and `TestStateDb` (local to main.rs)
- Implements `StateDb` and `revm::Database` traits for `TestStateDb`.
- Full EVM execution with state root progression
- Wiring: Starts consensus engine, then initializes and starts the JSON-RPC server.
- See `crates/whirlpool-node.md` for binary details

## Test Statistics

| Module | Test Count |
|--------|-----------|
| config.rs | 0 |
| main.rs (bin) | 0 |
| **Total (lib)** | **0** |


