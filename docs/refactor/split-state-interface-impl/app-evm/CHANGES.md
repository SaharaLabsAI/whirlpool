# CHANGES - app-evm

## Key Decisions

- **Grounded**: Keep interface trait bounds on `state::traits::StateDb`.
- **Assumption**: `app-evm` only needs concrete import-path rewiring for `InMemoryStateDb`; execution logic remains unchanged.
- **Rejected alternative**: Introduce temporary local wrapper types in `app-evm` to hide path changes. Rejected because it adds unnecessary adapter complexity.
- **Rationale**: Direct import migration keeps behavior stable and limits risk to compile-time dependency/path churn.

## Current State

- `crates/app-evm/src/executor.rs` and tests import/use `state::InMemoryStateDb`.
- `crates/app-evm/src/traits.rs` uses `state::traits::StateDb` for `StateProvider` bridging.

## Proposed Changes

- Switch concrete imports from `state::InMemoryStateDb` to `state_memory::InMemoryStateDb`.
- Keep trait-path usage on `state::traits::StateDb` unchanged.
- Add `state-memory` dependency in `crates/app-evm/Cargo.toml`.
- Update tests and fixtures to instantiate concrete DB from `state-memory`.

## Impact on Dependents

- `app-evm` continues to provide the same public behavior and trait bounds.
- Compile-time failures are expected if any old concrete path remains.

## Migration Notes

- File-level edits expected in:
  - `crates/app-evm/src/executor.rs`
  - `crates/app-evm/src/traits.rs` (imports only if needed)
  - `crates/app-evm/tests/*`
  - `crates/app-evm/Cargo.toml`
- Preserve decode/propose/verify execution semantics.
