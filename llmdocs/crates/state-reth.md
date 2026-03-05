# state-reth

## Purpose
Persistent state storage implementation backed by reth-db (MDBX/libmdbx).

## Modules
- `crates/state-reth/src/db.rs` — `RethStateDb` implementation, revm `Database`/`DatabaseRef` impls
- `crates/state-reth/src/init.rs` — `open_state_db` helper
- `crates/state-reth/src/error.rs` — `RethStateError` enum
- `crates/state-reth/src/tables.rs` — reth-db table re-exports
- `crates/state-reth/src/trie.rs` — Keccak256 state root computation
- `crates/state-reth/src/codec.rs` — account/info serialization

## Key Types
- `RethStateDb`: persistent DB wrapping `Arc<DatabaseEnv>`. Thread-safe, implements `StateDb`, `Database`, and `DatabaseRef`.
- `RethStateError`: error type with `Database`, `Init`, `Codec`, and `StateRoot` variants. Implements `DBErrorMarker`.

## Public API
- `open_state_db(path: &Path) -> Result<RethStateDb, RethStateError>`: primary entry point.
- `RethStateDb::open(path: &Path) -> Result<Self, RethStateError>`: opens or creates MDBX environment.

## Trait Implementations
- `state::traits::StateDb`: persistent implementation with dual-writes (Plain + Hashed tables).
- `revm::Database`: mutable EVM database access (per-call transactions).
- `revm::DatabaseRef`: read-only EVM database access.

## Internal Design
- **Persistence**: uses `reth_db::DatabaseEnv` (MDBX).
- **Dual-Writes**: state is written to both Plain and Hashed tables to support both direct lookups and Merkle trie generation.
- **Trie**: Keccak256 hashing for state root computation via `compute_state_root`.
- **Concurrency**: `Arc<DatabaseEnv>` enables `Clone + Send + Sync`.

## Tables Used
- `PlainAccountState`: address -> account info
- `PlainStorageState`: address -> slot -> value
- `Bytecodes`: code hash -> bytecode
- `HashedAccounts`: hashed address -> account info
- `HashedStorages`: hashed address -> hashed slot -> value
- `CanonicalHeaders`: block number -> block hash

## Canonical Imports
- `state_reth::RethStateDb`
- `state_reth::RethStateError`
- `state_reth::open_state_db`

## Dependencies
- `state`: interface traits and error types
- `reth-db`: MDBX database environment and transactions
- `reth-db-api`: table definitions and cursors
- `revm`: database traits and primitives
- `alloy-primitives`: hashing and Ethereum types
- `thiserror`: error derivation

## Test Coverage
- 20 total tests (13 unit + 7 integration)
- Coverage: persistence, concurrency, genesis allocation, deterministic state root, revm trait compatibility.

## Status
Complete. Production-ready persistent state implementation for Whirlpool nodes.
