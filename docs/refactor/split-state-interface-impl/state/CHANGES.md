# CHANGES - state

## Key Decisions

- **Grounded**: Keep `StateDb`, `StateError`, and `DBErrorMarker` in `state` as stable interface/error contracts.
- **Assumption**: `state` can remain interface-first even if it still depends on `revm` types used in trait signatures.
- **Rejected alternative**: Move `StateError` into `state-memory`. Rejected because `revm` database contracts in multiple crates require one shared error anchor.
- **Rationale**: A stable interface crate minimizes downstream trait-bound churn while implementation symbols relocate.

## Current State

- `crates/state/src/lib.rs` currently re-exports both interface and concrete symbols.
- `crates/state/src/traits.rs` defines `StateDb` and is used by `app-evm` trait bounds.
- `crates/state/src/error.rs` defines `StateError` and `DBErrorMarker`.
- `crates/state/src/db.rs` still contains concrete `DbAccount`/`InMemoryStateDb` and `revm` impls pre-split.

## Proposed Changes

- Keep `StateDb` and `StateError` canonical in `state`.
- Remove concrete `DbAccount`/`InMemoryStateDb` exports from `state` after consumers migrate.
- Keep `DBErrorMarker` implementation in `state` so concrete crates can keep using `StateError`.
- Restrict `state` crate root to interface/shared exports only.

## Impact on Dependents

- Interface-only consumers continue to depend only on `state`.
- Concrete consumers (`app-evm`, `whirlpool-node`) stop importing concrete DB types from `state`.

## Migration Notes

- File-level edits expected in:
  - `crates/state/src/lib.rs`
  - `crates/state/src/traits.rs`
  - `crates/state/src/error.rs`
- Keep trait signatures and error semantics unchanged.
