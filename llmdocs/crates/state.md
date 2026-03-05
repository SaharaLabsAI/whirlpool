# state

## Purpose
State database trait and error types — pure interface crate with no concrete implementations.

## Crate Split
`state` is the interface half of the state layer. Concrete implementations live in `state-memory`.

## Modules
- `crates/state/src/traits.rs` — `StateDb` trait
- `crates/state/src/error.rs` — `StateError` enum, `DBErrorMarker` impl

## Trait Boundary
`StateDb` defines the crate-level database contract:
- `type Error`: fallible operations associated error type.
- constructors: `new`, `with_genesis`
- state transitions: `commit` (returns `Result`)
- queries: `state_root`, account/code/storage/block-hash accessors (return `Result`)
- mutation helpers: `insert_account`, `insert_block_hash` (return `Result`)

## Canonical Imports
- `state::traits::StateDb`
- `state::StateDb` (re-export)
- `state::StateError`
- `state::GenesisAccount` (re-export from `alloy-genesis`)

## Dependencies
- `revm` (trait parameter types, `DBErrorMarker`)
- `alloy-genesis` (`GenesisAccount` in trait signature)
- `thiserror` (error derive)

## Status
Complete. Interface-only crate after physical split. See `state-memory` for concrete implementation.
