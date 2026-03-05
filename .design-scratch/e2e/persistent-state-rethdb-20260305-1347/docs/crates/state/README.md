# state

## Purpose / Overview

`state` remains the backend-agnostic contract crate for shared execution and RPC state access. In this iteration, it is modified only to define canonical fallible `StateDb` signatures so persistent MDBX-backed implementations can propagate I/O and codec failures safely.

This document covers **changes only** relative to current trait behavior.

## Public API Surface (Changed)

### `StateDb` trait (fallibility migration)

```rust
pub trait StateDb {
    type Error: std::error::Error + Send + Sync + 'static;

    fn new() -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn with_genesis(
        alloc: std::collections::HashMap<revm::primitives::Address, alloy_genesis::GenesisAccount>,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn state_root(&self) -> Result<revm::primitives::B256, Self::Error>;
    fn commit(&mut self, bundle: &revm::database::BundleState) -> Result<(), Self::Error>;

    fn get_account(
        &self,
        address: revm::primitives::Address,
    ) -> Result<Option<revm::state::AccountInfo>, Self::Error>;

    fn get_code_by_hash(
        &self,
        code_hash: revm::primitives::B256,
    ) -> Result<revm::state::Bytecode, Self::Error>;

    fn get_storage(
        &self,
        address: revm::primitives::Address,
        index: revm::primitives::U256,
    ) -> Result<revm::primitives::U256, Self::Error>;

    fn get_block_hash(&self, number: u64) -> Result<revm::primitives::B256, Self::Error>;

    fn insert_account(
        &mut self,
        address: revm::primitives::Address,
        info: revm::state::AccountInfo,
    ) -> Result<(), Self::Error>;

    fn insert_block_hash(
        &mut self,
        number: u64,
        hash: revm::primitives::B256,
    ) -> Result<(), Self::Error>;
}
```

### Existing public exports remain

```rust
pub use alloy_genesis::GenesisAccount;
pub use error::StateError;
pub use traits::StateDb;
```

## Internal Module Structure

- `traits.rs`: updated `StateDb` signatures with associated error type.
- `error.rs`: existing shared error marker surface (`StateError`) retained for compatibility where appropriate.
- `lib.rs`: unchanged re-export surface.

## Dependencies

### Internal workspace

- none

### External

- `revm`
- `alloy-genesis`
- `thiserror`

## Error Types and Error Handling Strategy

- The trait now delegates error domain ownership to implementations via associated `type Error`.
- Required bounds (`Error + Send + Sync + 'static`) guarantee cross-thread and long-lived context compatibility in app/runtime layers.
- No panics or silent fallback are part of the contract for storage operations; implementers must propagate errors via `Result`.

## Thread Safety / Concurrency Guarantees

- `StateDb` itself does not enforce synchronization primitives.
- Contract is designed for implementations used behind `Arc<RwLock<S>>` in consumers requiring `S: Send + Sync + 'static`.
- Associated error bounds are explicitly thread-safe to support shared runtime/RPC propagation.

## Constructor Patterns

- Constructors remain trait-level (`new`, `with_genesis`) but are now fallible.
- Implementations may provide additional constructors (for path/config/env injection) outside trait.

## Key Invariants

- Method names and conceptual behavior remain stable across migration.
- Return-value semantics are preserved; only error channel is introduced.
- Implementers must keep `state_root`, account/storage/code/block-hash behavior consistent with previous meaning unless explicitly documented by backend-specific contracts.

## Migration Notes (Required)

### Backward compatibility contract

- Source compatibility is intentionally broken for direct callers of old infallible methods.
- Behavioral compatibility is retained by wrapping previous returns in `Ok(...)` for infallible backends.

### `state-memory` migration path

- Add `type Error = core::convert::Infallible` (or crate-local never-fails error type).
- Update each `StateDb` method to return `Result<_, Self::Error>` with existing logic wrapped in `Ok(...)`.
- Keep `revm::Database` and `revm::DatabaseRef` error type as existing `StateError` (or align intentionally if refactoring later).
- Do not change deterministic in-memory state-root algorithm in this migration step.

### Consumer migration expectations

- `app-evm` and `rpc-eth` must propagate `StateDb::Error` at call sites that currently assume infallible trait reads/writes.
- Error mapping to execution/RPC surfaces is a consumer responsibility and must not be hard-coded in `state`.

## Blocker Resolution Notes

- **BLK-001 resolved:** canonical fallible trait contract is now pinned (associated `Error` + `Result` returns for all methods).
- **BLK-002 delegated:** trie-root semantics remain backend-specific (`state-reth` contract).
- **BLK-003 delegated:** host/runtime prerequisites are enforced at wiring/runtime layers, not trait layer.
