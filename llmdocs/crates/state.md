# state

## Purpose
State database abstractions and in-memory implementation for EVM execution.

## Interface/Implementation Split
- Interface module: `crates/state/src/traits.rs`
  - `StateDb`
- Implementation module: `crates/state/src/db.rs`
  - `InMemoryStateDb`

## Trait Boundary
`StateDb` defines the crate-level database contract:
- constructors: `new`, `with_genesis`
- state transitions: `commit`
- queries: `state_root`, account/code/storage/block-hash accessors
- mutation helpers: `insert_account`, `insert_block_hash`

`InMemoryStateDb` implements `StateDb` and also implements revm database traits used by execution.

## Canonical Imports
- `state::traits::StateDb`
- `state::InMemoryStateDb`

## Key Types
- `DbAccount`: account + storage container.
- `StateError`: state database error type.

## Status
Complete. Public interface is separated from implementation.
