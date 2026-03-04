# CHANGES — consensus-simplex

## Key Decisions

- **Grounded**: `CommonwareBlock` must move to a dedicated interface module.
- **[PROPOSED]**: Keep temporary compatibility exports due to high-risk generic-bound usage.

## Current State

- `crates/consensus-simplex/src/types.rs` defines `CommonwareBlock` trait and blanket impl.
- `adapter.rs` and `engine.rs` import `CommonwareBlock` from `crate::types`, and generic bounds are tightly coupled.
- `crates/consensus-simplex/src/lib.rs` re-exports `CommonwareBlock` from `types`.

## Proposed Changes

- **Grounded**: Extract `CommonwareBlock` into explicit interface module `crates/consensus-simplex/src/traits.rs`.
- **[PROPOSED]**: Keep compatibility export from `types` (or crate root) during migration to avoid generic-bound path breakage.
- **[PROPOSED]**: Update internal imports in `adapter.rs`, `engine.rs`, and tests to canonical `crate::traits::CommonwareBlock` first, before cross-crate consumers.
- **[PROPOSED]**: Keep trait semantics and blanket impl unchanged.

## Impact on Dependents

- `CommonwareEngine` and `AppAdapter` bounds remain behaviorally identical, but import paths shift.
- Downstream crates compile during transition because old and new paths are temporarily available.

## Migration Notes

- File-level edits:
  - `crates/consensus-simplex/src/traits.rs`: new home for trait + blanket impl.
  - `crates/consensus-simplex/src/types.rs`: remove or compatibility-re-export moved trait.
  - `crates/consensus-simplex/src/{adapter.rs,engine.rs,lib.rs,tests.rs}`: path updates to canonical module.
- Treat this as high-risk step; verify with targeted crate tests.
