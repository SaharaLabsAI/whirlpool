# CHANGES — state

## Key Decisions

- **Grounded**: Introduce a local `StateDb` trait to separate interface from concrete DB implementation.
- **[PROPOSED]**: Keep introduction additive with existing `InMemoryStateDb` exports preserved.

## Current State

- `crates/state/src/db.rs` contains `InMemoryStateDb` concrete implementation plus `Database`/`DatabaseRef` impls.
- `crates/state/src/lib.rs` exposes concrete types and errors but has no local trait boundary.

## Proposed Changes

- **Grounded**: Introduce `state::traits::StateDb` as a local interface contract.
- **[PROPOSED]**: Add `crates/state/src/traits.rs` defining `StateDb` methods needed by dependents (`state_root`, `commit`).
- **[PROPOSED]**: Implement `StateDb` for `InMemoryStateDb` in `db.rs` (or an impl-focused module).
- **[PROPOSED]**: Update `crates/state/src/lib.rs` to export `StateDb` while preserving concrete exports (`InMemoryStateDb`, `DbAccount`).

## Impact on Dependents

- `app-evm` can depend on `state::StateDb` contract rather than concrete-only assumptions.
- Foundational dependency layering remains unchanged because `StateDb` stays interface-only and local to `state`.

## Migration Notes

- File-level edits:
  - `crates/state/src/traits.rs`: new trait definition.
  - `crates/state/src/db.rs`: trait implementation for `InMemoryStateDb`.
  - `crates/state/src/lib.rs`: module declaration + `pub use traits::StateDb;`.
- Keep runtime behavior and state transition logic unchanged.
