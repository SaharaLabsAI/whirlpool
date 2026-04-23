# state

## Purpose
State database trait and error types — pure interface crate with no concrete implementations.

## Crate Split
`state` is the interface half of the state layer. Concrete implementations now live in `app-evm-state`; its public test utility also re-exports `InMemoryStateDb` for fast non-persistent fixtures.

## Modules
- `crates/app/state/src/block_storage.rs` — `BlockStorage` trait + `BlockStorageError`
- `crates/app/state/src/traits.rs` — `StateDb` trait
- `crates/app/state/src/error.rs` — `StateError` enum, `DBErrorMarker` impl

## Trait Boundary
`StateDb` defines the crate-level database contract:
- `type Error`: fallible operations associated error type.
- constructors: `new`, `with_genesis`
- state transitions: `commit` (returns `Result`)
- queries: `state_root`, account/code/storage/block-hash accessors (return `Result`)
- mutation helpers: `insert_account`, `insert_storage`, `insert_block_hash` (return `Result`)

`BlockStorage` defines the contract for persistent block and receipt storage:
- `store_block(&self, block: &EvmBlock, receipts: &[Receipt]) -> Result<(), BlockStorageError>`: Atomic persistence.
- `get_latest_block_number(&self) -> Result<Option<u64>, BlockStorageError>`: Returns highest stored block number or None if empty.
- `get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>, BlockStorageError>`
- `get_block_by_hash(&self, hash: B256) -> Result<Option<EvmBlock>, BlockStorageError>`
- `get_receipts_by_block(&self, number: u64) -> Result<Option<Vec<Receipt>>, BlockStorageError>`

## Canonical Imports
- `state::traits::StateDb`
- `state::StateDb` (re-export)
- `state::BlockStorage` (re-export)
- `state::BlockStorageError` (re-export)
- `state::StateError`
- `state::GenesisAccount` (re-export from `alloy-genesis`)

## Dependencies
- `revm` (trait parameter types, `DBErrorMarker`)
- `alloy-genesis` (`GenesisAccount` in trait signature)
- `thiserror` (error derive)

## Status
Complete. Interface-only crate after physical split. See `app-evm-state` for both the persistent MDBX backend and the shared in-memory test DB.
