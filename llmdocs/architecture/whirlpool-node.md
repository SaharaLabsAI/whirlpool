# Whirlpool Node Library: Config Constants

## Overview

The `whirlpool-node` crate library exports shared configuration constants used by both node binaries (`whirlpool-node` EVM and `whirlpool-node-simple` non-EVM).

Previously this crate also exported `EmptyBlock` and `EmptyBlockApp` types — these have been moved to `whirlpool-node-simple` as local modules.

## Library Exports

`crates/whirlpool-node/src/lib.rs`: `pub mod config;`

### config.rs
- `pub const NAMESPACE: &[u8]` — `b"sahara-chain-v0"`
- `pub const BLOCK_INTERVAL: Duration` — 5 seconds
- `pub const VALIDATOR_SEED: [u8; 32]` — `[0u8; 32]`
- `pub const BIND_ADDR: &str` — `"127.0.0.1:0"`

## Dependency Graph

- **whirlpool-node** (lib) → used by `whirlpool-node` (bin) and `whirlpool-node-simple` (bin) for config constants

## Binary: whirlpool-node (EVM)

Location: `crates/whirlpool-node/src/main.rs`
- Uses `EvmApplication` (from `app-evm`) with `InMemoryTxPool` (from `app`) and `TestStateDb` (local to main.rs)
- Full EVM execution with state root progression
- See `crates/whirlpool-node.md` for binary details

## Binary: whirlpool-node-simple (Non-EVM)

Location: `crates/whirlpool-node-simple/src/main.rs`
- Uses local `EmptyBlockApp` and `EmptyBlock` types (defined in `whirlpool-node-simple/src/app.rs` and `block.rs`)
- Pure consensus without execution layer
- See `crates/whirlpool-node-simple.md` for binary details

## Test Statistics

| Module | Test Count |
|--------|-----------|
| config.rs | 0 |
| main.rs (bin) | 0 |
| **Total (lib)** | **0** |

Note: EmptyBlock/EmptyBlockApp tests (18 total) now live in `whirlpool-node-simple`. Integration tests for consensus wiring with EmptyBlockApp have been removed from this crate.
