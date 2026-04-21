# state-memory

## Purpose
In-memory state database implementation for EVM execution.

## Location
`crates/app/execute/mem/state/`

## Crate Split
`state-memory` is the implementation half of the state layer. Interface traits live in `state`.

## Modules
- `crates/app/execute/mem/state/src/db.rs` — `InMemoryStateDb`, `DbAccount`, revm `Database`/`DatabaseRef` impls
- `crates/app/execute/mem/state/src/db_api_write.rs` — infallible write wrappers (`commit`, `insert_account`, `insert_storage`)
- `crates/app/execute/mem/state/src/db_api_account.rs` — infallible account/storage read wrappers (`get_account`, `get_code_by_hash`, `get_storage`)
- `crates/app/execute/mem/state/src/db_api_state.rs` — infallible state/hash wrappers (`state_root`, `insert_block_hash`, `get_block_hash`)
- `crates/app/execute/mem/state/src/tests/db.rs` — file-separated `db.rs` unit tests (wired via `#[path = "tests/db.rs"] mod tests;`)
- `crates/app/execute/mem/state/src/personality.rs` — `InMemoryPersonalityStorage` and in-memory finalized personality indexes

## Key Types
- `InMemoryStateDb`: HashMap-backed state DB implementing `StateDb`, `Database`, `DatabaseRef`, and in-memory `BlockStorage`.
- `DbAccount`: account info + storage container.
- `InMemoryPersonalityStorage`: in-memory `PersonalityStorage` implementation used outside `whirlpool-node`.

## Trait Implementations
- `state::traits::StateDb`: `type Error = Infallible`. All trait methods return `Result<_, Infallible>`.
- `revm::Database`: mutable EVM database access.
- `revm::DatabaseRef`: read-only EVM database access.
- `state::BlockStorage`: finalized block + receipt persistence via internal in-memory maps (`blocks_by_number`, `receipts_by_block`).
- `state::PersonalityStorage`: finalized personality lookups keyed by personality ID, tx hash, and signer/nonce.

## Canonical Imports
- `state_memory::InMemoryStateDb`
- `state_memory::DbAccount`
- `state_memory::InMemoryPersonalityStorage`

## Runtime Notes
- `insert_storage(address, slot, value)` now mutates in-memory storage directly (`value == 0` deletes slot), enabling app-layer precompile ledger writes in tests and local execution.

## Status
Active. Moved under `crates/app/execute/mem/` while keeping package/import names unchanged.
