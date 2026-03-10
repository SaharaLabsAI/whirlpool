# state-reth

## Purpose
Persistent state storage implementation backed by reth-db (MDBX/libmdbx).

## Modules
- `crates/state-reth/src/db.rs` — `RethStateDb` implementation, revm `Database`/`DatabaseRef` impls
- `crates/state-reth/src/block_storage.rs` — `BlockStorage` persistence for finalized blocks + receipts
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
- `state::block_storage::BlockStorage`: finalized block/receipt persistence and recovery (`store_block`, `get_latest_block_number`, `get_block_by_number`, `get_block_by_hash`, `get_receipts_by_block`).
- `revm::Database`: mutable EVM database access (per-call transactions).
- `revm::DatabaseRef`: read-only EVM database access.

## Internal Design
- **Persistence**: uses `reth_db::DatabaseEnv` (MDBX).
- **Dual-Writes**: state is written to both Plain and Hashed tables to support both direct lookups and Merkle trie generation.
- **Block Storage**: `store_block` validates tx/receipt length before opening a write tx, decodes 2718 txs once, writes canonical header/body/tx/receipt records atomically, and treats identical `(number, hash)` re-inserts as idempotent no-ops. `get_latest_block_number` uses `cursor_read::<CanonicalHeaders>().last()` for O(log N) tip recovery.
- **Trie**: Keccak256 hashing for state root computation via `compute_state_root`.
- **Concurrency**: `Arc<DatabaseEnv>` enables `Clone + Send + Sync`.

## Tables Used
- `PlainAccountState`: address -> account info
- `PlainStorageState`: address -> slot -> value
- `Bytecodes`: code hash -> bytecode
- `HashedAccounts`: hashed address -> account info
- `HashedStorages`: hashed address -> hashed slot -> value
- `CanonicalHeaders`: block number -> block hash (used for `get_latest_block_number`)
- `Headers`: block number -> header
- `HeaderNumbers`: block hash -> block number
- `HeaderTerminalDifficulties`: block number -> total difficulty wrapper
- `BlockBodyIndices`: block number -> first tx index + tx count
- `Transactions`: tx number -> signed tx
- `TransactionHashNumbers`: tx hash -> tx number
- `TransactionBlocks`: last tx number in block -> block number
- `Receipts`: tx number -> receipt

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
- 30 total tests (23 unit + 7 integration).
- Block storage unit tests: TC-SR-01..10 in `crates/state-reth/src/block_storage.rs`.
- Coverage: persistence, recovery (TC-SR-09/10), concurrency, genesis allocation, deterministic state root, revm trait compatibility, block/receipt persistence round-trips.

## Status
Complete. Production-ready persistent state implementation for Whirlpool nodes.
