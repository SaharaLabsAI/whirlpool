# CHANGES — app-evm

## Key Decisions

- **Grounded**: `StateProvider` must be split from executor implementation into a trait module.
- **[PROPOSED]**: Preserve executor-level compatibility export while downstream imports migrate.

## Current State

- `crates/app-evm/src/executor.rs` contains both `StateProvider` trait definition and `EvmApplication` implementation.
- `InMemoryStateDb` trait impl for `StateProvider` is also colocated in `executor.rs`.
- `crates/app-evm/src/lib.rs` does not expose a dedicated interface module.

## Proposed Changes

- **Grounded**: Move `StateProvider` contract into a dedicated `crates/app-evm/src/traits.rs` interface module.
- **[PROPOSED]**: Keep temporary compatibility export from `executor.rs` during migration.
- **[PROPOSED]**: Update `crates/app-evm/src/lib.rs` to expose `pub mod traits;` and `pub use traits::StateProvider;`.
- **[PROPOSED]**: Keep `EvmApplication<DB: StateProvider + ...>` bounds and executor behavior unchanged; only symbol location/import path changes.

## Impact on Dependents

- Call sites importing `StateProvider` from `app-evm::executor` can migrate to `app-evm::traits` incrementally.
- `state` boundary work (`state::traits::StateDb`) and `app-evm` boundary become cleaner without runtime semantic changes.

## Migration Notes

- File-level edits:
  - `crates/app-evm/src/traits.rs`: new `StateProvider` trait home.
  - `crates/app-evm/src/executor.rs`: remove local trait definition, import canonical trait, keep compatibility re-export as needed.
  - `crates/app-evm/src/lib.rs`: add module/export wiring.
- Preserve decode/propose/verify execution paths exactly.
