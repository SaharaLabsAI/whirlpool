# app-evm-state

## Purpose
Persistent state storage implementation backed by reth-db (MDBX/libmdbx), plus the shared in-memory test DB used after `state-memory` crate removal.

## Modules
- `crates/app/evm-state/src/db.rs` — `RethStateDb` core implementation + `StateDb` impl
- `crates/app/evm-state/src/db_revm_impls.rs` — `revm::Database` / `revm::DatabaseRef` adapter impls
- `crates/app/evm-state/src/tests/db.rs` — file-separated `db.rs` unit tests (wired via `#[path = "tests/db.rs"] mod tests;`)
- `crates/app/evm-state/src/block_storage.rs` — `BlockStorage` persistence for finalized blocks + receipts; compiled as a private child of `db.rs` so it can use storage internals without public/raw handles
- `crates/app/evm-state/src/dkg_history.rs` — `app_primitives::header_extra_data::HeaderExtraDataHistory` implementations for raw historical header `extra_data` bytes
- `crates/app/evm-state/src/tests/block_storage.rs` — file-separated block-storage unit tests (wired via `#[path = "tests/block_storage.rs"] mod tests;`)
- `crates/app/evm-state/src/init.rs` — `open_state_db` helper
- `crates/app/evm-state/src/error.rs` — `RethStateError` enum
- `crates/app/evm-state/src/rpc_reader.rs` — narrow semantic read API used by `rpc-eth` instead of exposing raw tables/transactions
- `crates/app/evm-state/src/trie.rs` — state root computation via `StateRoot<DatabaseTrieCursorFactory<_, LegacyKeyAdapter>, ...>`
- `crates/app/evm-state/src/codec.rs` — account/info serialization
- `crates/app/evm-state/src/in_memory_db.rs` — `InMemoryStateDb` test utility migrated from the removed `state-memory` crate

## Key Types
- `RethStateDb`: persistent DB wrapping `Arc<DatabaseEnv>`. Thread-safe, implements `StateDb`, `Database`, and `DatabaseRef`.
- `InMemoryStateDb`: HashMap-backed `StateDb` + `BlockStorage` test utility re-exported from `app-evm-state` so other crates no longer depend on a separate `state-memory` package.
- `RethStateError`: error type with `Database`, `Init`, `Codec`, and `StateRoot` variants. Implements `DBErrorMarker`.

## Public API
- `open_state_db(path: &Path) -> Result<RethStateDb, RethStateError>`: primary entry point.
- `RethStateDb::open(path: &Path) -> Result<Self, RethStateError>`: opens or creates MDBX environment.
- `RethStateDb::rpc_reader() -> RpcStateReader`: semantic read projection for current `rpc-eth` needs (headers, bodies, txs, receipts, account lookup) without exposing raw table handles or DB transactions.
- `RethStateDb::apply_genesis(chain_spec: &ChainSpec) -> Result<(), RethStateError>`: writes genesis account state (balance, nonce, code, storage) from the chain spec's genesis allocations into MDBX. Writes to both Plain and Hashed tables. Used by `start_node_with_chain_spec()` for integration tests with pre-funded accounts.

## Trait Implementations
- `state::traits::StateDb`: persistent implementation with dual-writes (Plain + Hashed tables).
- `state::block_storage::BlockStorage`: finalized block/receipt persistence and recovery (`store_block`, `get_latest_block_number`, `get_block_by_number`, `get_block_by_hash`, `get_receipts_by_block`).
- `app_primitives::header_extra_data::HeaderExtraDataHistory`: raw historical header-byte lookup. `RethStateDb` reads `Headers.extra_data` directly without reconstructing `EvmBlock`; `InMemoryStateDb` clones stored `EvmBlock.extra_data`. Neither implementation decodes header bytes or validates DKG metadata.
- `revm::Database`: mutable EVM database access (per-call transactions).
- `revm::DatabaseRef`: read-only EVM database access.

## Internal Design
- **Persistence**: uses `reth_db::DatabaseEnv` (MDBX).
- **Dual-Writes**: state is written to both Plain and Hashed tables to support both direct lookups and Merkle trie generation.
- **Storage writer seam**: `insert_storage` writes/deletes storage slots in both Plain and Hashed tables, creating an empty account row when needed so slot ownership is canonical.
- **Block Storage**: `store_block` validates tx/receipt length before opening a write tx, decodes 2718 txs once, writes canonical header/body/tx/receipt records atomically, and treats identical `(number, hash)` re-inserts as idempotent no-ops. Stored headers include `base_fee_per_gas`, proposer fee recipient in `beneficiary`, proposer public key in `extra_data`, and post-Cancun blob gas fields (`excess_blob_gas: Some(0)`, `blob_gas_used: Some(0)`). `get_latest_block_number` uses `cursor_read::<CanonicalHeaders>().last()` for O(log N) tip recovery.
- **Block reconstruction strictness**: `get_block_by_number` fails with `BlockStorageError::Codec` when proposer pubkey cannot be decoded from persisted `extra_data` or when required persisted `base_fee_per_gas` is absent. `HeaderExtraDataHistory` intentionally bypasses this reconstruction path and returns raw header `extra_data` bytes so execution can run the bounded historical scan while app-primitives decodes the carrier and validators-dkg interprets only DKG payload semantics.
- **Trie**: state root computation uses explicit `LegacyKeyAdapter` wiring for `StateRoot::from_tx(...)` compatibility with reth v2 trie-db generics.
- **Delete error propagation**: commit and `insert_storage` delete paths propagate MDBX delete errors instead of ignoring them. The former production-path test injection module was removed so the crate no longer carries failure-control hooks.
- **RPC storage boundary**: downstream crates do not import `app_evm_state::tables` or call `RethStateDb::inner()`; `rpc-eth` consumes `RpcStateReader` DTO/query methods instead. Reth table names are imported only inside private storage modules.
- db helper modules now use explicit imports instead of wildcard parent imports for clearer dependency boundaries.
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
- `app_evm_state::RethStateDb`
- `app_evm_state::InMemoryStateDb`
- `app_evm_state::RethStateError`
- `app_evm_state::open_state_db`

## Dependencies
- `state`: interface traits and error types
- `reth-db`: MDBX database environment and transactions
- `reth-db-api`: table definitions and cursors
- `revm`: database traits and primitives
- `alloy-primitives`: hashing and Ethereum types
- `thiserror`: error derivation

## Test Coverage
- 35 total tests (28 unit + 7 integration).
- Block storage unit tests: TC-SR-01..10 plus strict reconstruction regressions in `crates/app/evm-state/src/tests/block_storage.rs`; DKG history raw-carrier tests in `crates/app/evm-state/src/tests/dkg_history.rs`.
- Coverage: persistence, recovery (TC-SR-09/10), concurrency, genesis allocation, deterministic state root, revm trait compatibility, block/receipt persistence round-trips.

## Status
Complete. Production-ready persistent state implementation for Whirlpool nodes, and the canonical home for the shared in-memory test DB after `state-memory` removal.
Clippy hygiene: receipt mapping in `block_storage` now forwards `cumulative_gas_used` without redundant same-type casts.
