# state-memory

## Purpose
In-memory state database implementation for EVM execution.

## Crate Split
`state-memory` is the implementation half of the state layer. Interface traits live in `state`.

## Modules
- `crates/state-memory/src/db.rs` — `InMemoryStateDb`, `DbAccount`, revm `Database`/`DatabaseRef` impls

## Key Types
- `InMemoryStateDb`: HashMap-backed state DB implementing `StateDb`, `Database`, and `DatabaseRef`.
- `DbAccount`: account info + storage container.

## Trait Implementations
- `state::traits::StateDb` — full implementation with genesis loading, commit, state queries
- `revm::Database` — mutable EVM database access
- `revm::DatabaseRef` — read-only EVM database access

## Canonical Imports
- `state_memory::InMemoryStateDb`
- `state_memory::DbAccount`

## Dependencies
- `state` (interface traits and error types)
- `revm` (database traits, primitives)
- `alloy-genesis` (genesis account types)
- `sha2` (keccak256 for empty code hash)

## Status
Complete. Contains all concrete state implementations extracted from the former `state` crate.
