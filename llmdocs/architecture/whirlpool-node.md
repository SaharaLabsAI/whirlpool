# Whirlpool Node Library: Config Constants

## Overview

The `whirlpool-node` crate library exports shared configuration constants used by the EVM node binary.

## Library Exports

`crates/whirlpool-node/src/lib.rs`: `pub mod config;`

### config.rs
- `pub const NAMESPACE: &[u8]` — `b"sahara-chain-v0"`
- `pub const BLOCK_INTERVAL: Duration` — 5 seconds
- `pub const VALIDATOR_SEED: [u8; 32]` — `[0u8; 32]`
- `pub const BIND_ADDR: &str` — `"127.0.0.1:0"`

## Dependency Graph

- **whirlpool-node** (lib) → used by `whirlpool-node` (bin) for config constants

## Binary: whirlpool-node (EVM)

Location: `crates/whirlpool-node/src/main.rs`
- Uses `EvmApplication` (from `app-evm`) with `InMemoryTxPool` (from `app`) and `TestStateDb` (local to main.rs)
- Full EVM execution with state root progression
- See `crates/whirlpool-node.md` for binary details

## Test Statistics

| Module | Test Count |
|--------|-----------|
| config.rs | 0 |
| main.rs (bin) | 0 |
| **Total (lib)** | **0** |


