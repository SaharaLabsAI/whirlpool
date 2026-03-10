# CHANGES - state-memory

## Key Decisions

- **Grounded**: `state-memory` is the concrete implementation crate for in-memory DB behavior and `revm` integration.
- **Assumption**: Moving `DbAccount`, `InMemoryStateDb`, and both `revm` impls as one unit preserves runtime semantics.
- **Rejected alternative**: Split concrete types and `revm` impls across different crates/modules. Rejected because it increases coupling and relocation risk.
- **Rationale**: Co-locating storage structs and trait impls keeps implementation ownership explicit and reduces migration ambiguity.

## Current State

- Crate does not exist yet.
- All concrete state implementation currently lives in `crates/state/src/db.rs`.

## Proposed Changes

- Add `crates/state-memory` as a workspace crate.
- Move `DbAccount` and `InMemoryStateDb` into `state_memory::db`.
- Move `impl DatabaseRef for InMemoryStateDb` and `impl Database for InMemoryStateDb` into `state-memory`.
- Re-export `DbAccount` and `InMemoryStateDb` from crate root for ergonomic imports.

## Impact on Dependents

- `app-evm` and `whirlpool-node` will depend on `state-memory` for concrete DB usage.
- `state-memory` depends on `state` for `StateDb` and `StateError` contracts.

## Migration Notes

- File-level additions expected:
  - `crates/state-memory/Cargo.toml`
  - `crates/state-memory/src/lib.rs`
  - `crates/state-memory/src/db.rs`
- Preserve constructor behavior (`new`, `with_genesis`) and state transition logic exactly.
