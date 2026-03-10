# CHANGES — app

## Key Decisions

- **Grounded**: `Application` and `TxSource` remain in `app::traits` as the interface contract.
- **[PROPOSED]**: Move `NoopTxSource` and `InMemoryTxPool` to `app::tx_source` with compatibility re-exports.

## Current State

- `crates/app/src/traits.rs` contains both interface traits (`Application`, `TxSource`) and concrete implementations (`NoopTxSource`, `InMemoryTxPool`).
- `crates/app/src/lib.rs` re-exports all four symbols from `traits`, so interface and implementation boundaries are currently mixed.

## Proposed Changes

- **Grounded**: Keep `Application` and `TxSource` in `crates/app/src/traits.rs` as the canonical interface-only surface.
- **[PROPOSED]**: Add `crates/app/src/tx_source.rs` and move `NoopTxSource` + `InMemoryTxPool` there.
- **[PROPOSED]**: Update `crates/app/src/lib.rs` to declare `pub mod tx_source;` and re-export concrete types from `tx_source` while preserving old crate-root exports.
- **[PROPOSED]**: Move tx-source-specific tests from `traits.rs` into `tx_source.rs` (or split tests by module) to match ownership.

## Impact on Dependents

- `app-evm` and node wiring continue to compile through crate-root re-exports.
- Consumers can migrate gradually from `app::traits::{NoopTxSource, InMemoryTxPool}` to `app::tx_source::{...}` without immediate breakage.

## Migration Notes

- File-level edits:
  - `crates/app/src/traits.rs`: remove concrete tx-source structs/impls/tests; keep traits only.
  - `crates/app/src/tx_source.rs`: add concrete types, impls, and tx-source-specific tests.
  - `crates/app/src/lib.rs`: add module declaration and compatibility re-exports.
- Keep behavior identical (`pending()` drain semantics and mutex usage unchanged).
