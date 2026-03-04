# CHANGES — consensus

## Key Decisions

- **Grounded**: Canonical trait surface should be grouped under `consensus::traits`.
- **[PROPOSED]**: Preserve legacy module paths as temporary compatibility exports.

## Current State

- Trait definitions are distributed across implementation-oriented modules:
  - `crates/consensus/src/app.rs` (`ConsensusApp`)
  - `crates/consensus/src/block.rs` (`Block`)
  - `crates/consensus/src/event.rs` (`EventSink`)
  - `crates/consensus/src/engine.rs` (`ConsensusEngine`)
- `crates/consensus/src/lib.rs` re-exports traits directly from these modules, but no dedicated interface boundary exists.

## Proposed Changes

- **Grounded**: Introduce explicit `crates/consensus/src/traits.rs` as canonical interface module.
- **[PROPOSED]**: Move (or canonically re-export) `ConsensusApp`, `Block`, `EventSink`, and `ConsensusEngine` through `consensus::traits`.
- **[PROPOSED]**: Keep existing module paths (`consensus::app`, `consensus::block`, `consensus::event`, `consensus::engine`) as compatibility exports during migration.
- **[PROPOSED]**: Update `crates/consensus/src/lib.rs` so crate-root public API points to `traits` while preserving transitional compatibility.

## Impact on Dependents

- High downstream coupling in `consensus-simplex` generic bounds remains stable via temporary dual-path exports.
- Node and adapter crates can migrate imports to `consensus::traits::*` incrementally.

## Migration Notes

- File-level edits:
  - `crates/consensus/src/traits.rs`: new canonical trait surface.
  - `crates/consensus/src/lib.rs`: `pub mod traits;` and canonical re-exports.
  - `crates/consensus/src/{app.rs,block.rs,event.rs,engine.rs}`: retain compatibility visibility as needed.
- Do not change trait semantics, associated types, or async signatures.
