# app-evm

## Purpose
EVM configuration and execution integration for Whirlpool applications.

## Interface/Implementation Split
- Interface module: `crates/app-evm/src/traits.rs`
  - `StateProvider`
- Implementation modules:
  - `crates/app-evm/src/config.rs`
  - `crates/app-evm/src/executor.rs`
  - `crates/app-evm/src/error.rs`

## Trait Boundary
- `StateProvider` is now defined in `app_evm::traits`.
- Blanket impl delegates to `state::traits::StateDb`.

## Canonical Imports
- `app_evm::traits::StateProvider`
- `state::traits::StateDb` (interface trait)
- `state_memory::InMemoryStateDb` (concrete impl, test code only)

## Key Types
- `WhirlpoolEvmConfig`: wrapper for EVM configuration.
- `EvmApplication`: application implementation that executes EVM blocks.
- `EvmAppError`: EVM application error type.

## Status
Partial. Trait surface is stable (`traits.rs`); execution implementation remains in progress.
