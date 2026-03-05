# CHANGES - whirlpool-node

## Key Decisions

- **Grounded**: Runtime wrapper `TestStateDb` should consume concrete DB from `state-memory` while error contract stays in `state`.
- **Assumption**: Node-level runtime behavior remains unchanged if `TestStateDb` delegation logic is untouched.
- **Rejected alternative**: Keep `state::InMemoryStateDb` compatibility re-export in `state` for node indefinitely. Rejected to enforce interface/implementation separation.
- **Rationale**: Node should depend explicitly on concrete crate for implementation and interface crate for contracts.

## Current State

- `crates/whirlpool-node/src/main.rs` uses `state::InMemoryStateDb` inside `TestStateDb`.
- `TestStateDb` uses `state::StateError` in `revm::Database` implementation.

## Proposed Changes

- Replace concrete DB import with `state_memory::InMemoryStateDb`.
- Keep `StateError` usage from `state` unchanged.
- Add `state-memory` dependency in `crates/whirlpool-node/Cargo.toml`.
- Keep wrapper behavior and delegation paths unchanged.

## Impact on Dependents

- Node wiring keeps current behavior with explicit implementation dependency.
- Any residual old-path import will cause compile failure in node crate.

## Migration Notes

- File-level edits expected in:
  - `crates/whirlpool-node/src/main.rs`
  - `crates/whirlpool-node/Cargo.toml`
- Preserve EVM wiring and runtime initialization semantics.
