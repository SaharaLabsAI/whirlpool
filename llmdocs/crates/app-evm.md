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
- `type Error`: fallible operations associated error type.
- Blanket impl delegates to `state::traits::StateDb`.
- `state_root` and `commit` return `Result<_, Self::Error>`.

## Error Handling
- `EvmAppError::State(String)` — wraps database and state-related errors.
- `From<Infallible>`: trivial conversion for `InMemoryStateDb`.
- `From<state::StateError>`: generic state error conversion.
- `From<state_reth::RethStateError>`: persistent state error conversion.

## Execution Implementation
The `EvmApplication` executor uses `.map_err(Into::into)` on all `StateProvider` calls to convert into `EvmAppError`.

## Canonical Imports
- `app_evm::traits::StateProvider`
- `state::traits::StateDb` (interface trait)
- `state_reth::RethStateDb` (persistent implementation)
- `state_memory::InMemoryStateDb` (test code only)

## Key Types
- `WhirlpoolEvmConfig`: wrapper for EVM configuration.
- `EvmApplication`: application implementation that executes EVM blocks.
- `EvmAppError`: EVM application error type.

## Status
Partial. Trait surface is stable (`traits.rs`); execution implementation remains in progress.
