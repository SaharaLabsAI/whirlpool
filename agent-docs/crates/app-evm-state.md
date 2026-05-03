# app-evm-state

## Purpose
Persistent state storage implementation backed by reth-db (MDBX/libmdbx), plus the shared in-memory test DB used after `state-memory` crate removal.

## Module ownership
- `crates/app/evm-state/src/reth/` — Reth/MDBX implementation family owner.
  - `reth/db.rs` — `RethStateDb`, `open`, `apply_genesis`, `StateDb` impl, `rpc_reader()` entrypoint.
  - `reth/block_storage.rs` — `BlockStorage` persistence for finalized blocks and receipts.
  - `reth/dkg_history.rs` — raw `HeaderExtraDataHistory` lookup for persisted header `extra_data`.
  - `reth/revm.rs` — `revm::Database` / `DatabaseRef` impls.
  - `reth/trie.rs` — state-root computation through reth trie-db.
  - `reth/rpc_reader/` — semantic read projection used by `rpc-eth`; facade shape is `blocks()`, `transactions()`, `accounts()` with narrower child readers.
- `crates/app/evm-state/src/memory/` — in-memory fixture implementation family owner.
  - `memory/db.rs` — `InMemoryStateDb`, `StateDb`, `BlockStorage`, `Database`, `DatabaseRef`, and raw block-extra-data history impls.
  - `memory/api/` — convenience inherent API wrappers for account/state/write operations.
- `codec.rs` — account/info serialization helpers shared by the Reth family.
- `error.rs` — `RethStateError`.
- `init.rs` — `open_state_db`.
- `lib.rs` — top-level public re-exports plus re-export-only compatibility modules `db` and `in_memory_db`.

## Key Types
- `RethStateDb`: persistent DB wrapping `Arc<DatabaseEnv>`. Thread-safe, implements `StateDb`, `Database`, `DatabaseRef`, `BlockStorage`, and `HeaderExtraDataHistory`.
- `InMemoryStateDb`: HashMap-backed `StateDb` + `BlockStorage` test utility re-exported from `app-evm-state` so other crates no longer depend on a separate `state-memory` package.
- `RethStateError`: error type with `Database`, `Init`, `Codec`, and `StateRoot` variants. Implements `DBErrorMarker`.
- `RpcStateReader`: semantic facade exposing exactly `blocks()`, `transactions()`, and `accounts()`; direct query behavior lives in child readers under `reth/rpc_reader/`.

## Public API
- `open_state_db(path: &Path) -> Result<RethStateDb, RethStateError>`: primary entry point.
- `RethStateDb::open(path: &Path) -> Result<Self, RethStateError>`: opens or creates MDBX environment.
- `RethStateDb::apply_genesis(alloc: &HashMap<Address, GenesisAccount>) -> Result<(), RethStateError>`: writes genesis account state into MDBX.
- `RethStateDb::rpc_reader() -> RpcStateReader`: semantic read projection for `rpc-eth` without exposing raw tables/transactions.
- Stable top-level imports: `app_evm_state::{RethStateDb, InMemoryStateDb, RethStateError, open_state_db, RpcStateReader}`.
- Compatibility imports: `app_evm_state::db::RethStateDb` and `app_evm_state::in_memory_db::InMemoryStateDb` are re-export-only shims.

## RPC reader API map
- `RpcStateReader::blocks()` -> `RpcBlockReader`.
  - `canonical()` -> block hash range and canonical tip queries.
  - `headers()` -> header lookup/range/raw-carrier readers.
  - `bodies()` -> block reconstruction and body-index queries.
- `RpcStateReader::transactions()` -> lookup, metadata, and receipt readers.
- `RpcStateReader::accounts()` -> account reader.

## Trait Implementations
- `state::traits::StateDb`: persistent implementation with dual-writes to Plain and Hashed tables.
- `state::block_storage::BlockStorage`: finalized block/receipt persistence and recovery.
- `app_primitives::header_extra_data::HeaderExtraDataHistory`: raw historical header-byte lookup. Reth reads `Headers.extra_data`; memory clones stored `EvmBlock.extra_data`.
- `revm::Database` and `revm::DatabaseRef`: EVM database access.

## Internal Design
- **Persistence**: uses `reth_db::DatabaseEnv` (MDBX).
- **Implementation families**: Reth/MDBX code lives under `reth/**`; reusable in-memory fixtures live under `memory/**`; parent `lib.rs` only re-exports family entrypoints.
- **Dual-writes**: state is written to both Plain and Hashed tables to support direct lookups and Merkle trie generation.
- **Storage writer seam**: Reth helpers write/delete storage slots in both Plain and Hashed tables and create empty account rows when needed so slot ownership is canonical.
- **Block storage**: `store_block` validates tx/receipt length, decodes 2718 txs once, writes canonical header/body/tx/receipt records atomically, and treats identical `(number, hash)` re-inserts as idempotent no-ops.
- **RPC storage boundary**: downstream crates do not import raw tables or call `RethStateDb::inner()`; `rpc-eth` consumes child reader APIs from `RpcStateReader`.

## Tables Used
- `PlainAccountState`, `PlainStorageState`, `Bytecodes`, `HashedAccounts`, `HashedStorages`
- `CanonicalHeaders`, `Headers`, `HeaderNumbers`, `HeaderTerminalDifficulties`
- `BlockBodyIndices`, `Transactions`, `TransactionHashNumbers`, `TransactionBlocks`, `Receipts`

## Tests
- Source-adjacent Reth tests: `src/reth/tests/{db,block_storage,dkg_history,genesis,persistence,trie,concurrency}.rs`.
- Source-adjacent memory tests: `src/memory/tests/dkg_history.rs`.
- Crate-local `crates/app/evm-state/tests/*.rs` was removed; current coverage remains 35 unit/module tests.

## Status
Complete. Production-ready persistent state implementation for Whirlpool nodes, with explicit Reth and memory implementation-family boundaries and narrow RPC reader surfaces.
