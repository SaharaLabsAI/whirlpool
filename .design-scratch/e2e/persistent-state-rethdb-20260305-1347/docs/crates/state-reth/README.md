# state-reth

## Purpose / Overview

`state-reth` is the persistent `StateDb` backend for Whirlpool, implemented on top of `reth-db` (MDBX) and designed to replace in-memory runtime state in node wiring. It is the concrete storage adapter that preserves the existing EVM/RPC generic boundaries while adding durable state persistence.

This crate also provides `revm::Database` and `revm::DatabaseRef` implementations so EVM execution can read/write state directly through the same backend.

## Public API Surface

### Crate root exports (`lib.rs`)

```rust
pub mod db;
pub mod tables;
pub mod codec;
pub mod trie;
pub mod init;
pub mod error;

pub use db::RethStateDb;
pub use error::RethStateError;
pub use init::{create_db, init_db};
```

### Public struct

```rust
pub struct RethStateDb {
    env: std::sync::Arc<reth_db::DatabaseEnv>,
    path: std::path::PathBuf,
    db_args: reth_db::DatabaseArguments,
}
```

### Constructor patterns

```rust
impl RethStateDb {
    pub fn open(
        path: impl AsRef<std::path::Path>,
        db_args: reth_db::DatabaseArguments,
    ) -> Result<Self, RethStateError>;

    pub fn from_env(
        path: std::path::PathBuf,
        env: std::sync::Arc<reth_db::DatabaseEnv>,
        db_args: reth_db::DatabaseArguments,
    ) -> Self;
}
```

### Initialization helpers

```rust
pub fn create_db(
    path: impl AsRef<std::path::Path>,
    args: reth_db::DatabaseArguments,
) -> Result<reth_db::DatabaseEnv, RethStateError>;

pub fn init_db(
    path: impl AsRef<std::path::Path>,
    args: reth_db::DatabaseArguments,
) -> Result<reth_db::DatabaseEnv, RethStateError>;
```

### Error type

```rust
#[derive(Debug, thiserror::Error)]
pub enum RethStateError {
    #[error("Database error: {0}")]
    Database(#[from] reth_storage_errors::db::DatabaseError),
    #[error("MDBX init/open error: {0}")]
    Init(String),
    #[error("Codec error: {0}")]
    Codec(String),
    #[error("Trie root error: {0}")]
    StateRoot(String),
}

impl revm::database::DBErrorMarker for RethStateError {}
```

### `StateDb` impl contract (11 entries = associated error + 10 methods)

```rust
impl state::StateDb for RethStateDb {
    type Error = RethStateError;

    fn new() -> Result<Self, Self::Error>;
    fn with_genesis(
        alloc: std::collections::HashMap<revm::primitives::Address, state::GenesisAccount>,
    ) -> Result<Self, Self::Error>;

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

### `revm::Database` impl contract (`&mut self`)

```rust
impl revm::Database for RethStateDb {
    type Error = RethStateError;

    fn basic(
        &mut self,
        address: revm::primitives::Address,
    ) -> Result<Option<revm::state::AccountInfo>, Self::Error>;

    fn storage(
        &mut self,
        address: revm::primitives::Address,
        index: revm::primitives::U256,
    ) -> Result<revm::primitives::U256, Self::Error>;

    fn block_hash(&mut self, number: u64) -> Result<revm::primitives::B256, Self::Error>;

    fn code_by_hash(
        &mut self,
        code_hash: revm::primitives::B256,
    ) -> Result<revm::state::Bytecode, Self::Error>;
}
```

### `revm::DatabaseRef` impl contract (`&self`)

```rust
impl revm::DatabaseRef for RethStateDb {
    type Error = RethStateError;

    fn basic_ref(
        &self,
        address: revm::primitives::Address,
    ) -> Result<Option<revm::state::AccountInfo>, Self::Error>;

    fn storage_ref(
        &self,
        address: revm::primitives::Address,
        index: revm::primitives::U256,
    ) -> Result<revm::primitives::U256, Self::Error>;

    fn block_hash_ref(&self, number: u64) -> Result<revm::primitives::B256, Self::Error>;

    fn code_by_hash_ref(
        &self,
        code_hash: revm::primitives::B256,
    ) -> Result<revm::state::Bytecode, Self::Error>;
}
```

## Internal Module Structure

- `db.rs`: `RethStateDb` type, `StateDb` impl, `revm::Database`/`DatabaseRef` impls.
- `tables.rs`: typed table access helpers and key translation.
- `codec.rs`: revm <-> reth data model conversions.
- `trie.rs`: state-root generation using trie overlay computation.
- `init.rs`: DB creation/open/init helpers and genesis initialization helpers.
- `error.rs`: `RethStateError` and conversion impls.

## Table Mapping Contract (`StateDb` -> reth-db)

- `get_account` / `insert_account` -> `PlainAccountState`.
- `get_storage` / storage writes in `commit` -> `PlainStorageState` (dupsort cursor).
- `get_code_by_hash` / code writes in `commit` -> `Bytecodes`.
- `state_root` reads account+storage tables and trie tables (`AccountsTrie`, `StoragesTrie`) through reth trie APIs.
- `with_genesis` seeds account/storage/code tables in one write transaction.
- `get_block_hash` / `insert_block_hash` -> contract target is `CanonicalHeaders` (resolved decision; keep implementation abstraction in `tables.rs`).

## State Root Contract

`state_root()` must compute canonical trie root via `reth_trie::StateRoot::overlay_root` over hashed post-state prepared from plain state tables.

Contract invariants:
- root source includes account leaf changes and storage leaf changes;
- zero-value storage slots are excluded from hashed-state input;
- account/code/storage normalization follows reth compact codec boundaries;
- root output is deterministic for equivalent logical state.

This resolves **BLK-002** at contract level by pinning source tables + normalization rules + algorithm (`overlay_root`).

## Dependencies

### Internal workspace

- `state`

### External / vendored

- `reth-db` (MDBX env + tx + table APIs)
- `reth-db-api`
- `reth-storage-errors`
- `reth-codecs`
- `reth-trie`
- `revm`
- `alloy-primitives`
- `thiserror`

## Error Handling Strategy

- All public operations are fallible via `Result<_, RethStateError>`.
- MDBX tx open/read/write/commit failures map to `RethStateError::Database` or `RethStateError::Init`.
- Codec translation failures map to `RethStateError::Codec`.
- Trie calculation failures map to `RethStateError::StateRoot`.
- Write failure policy: transaction is dropped/aborted; no partial commit visibility.

## Thread Safety and Concurrency Guarantees

- `RethStateDb` is `Send + Sync` by composition (`Arc<DatabaseEnv>` + `PathBuf` + immutable args).
- Clone strategy: `Clone` duplicates only `Arc<DatabaseEnv>` and metadata, sharing one MDBX environment.
- MDBX concurrency model:
  - read methods acquire short-lived read tx per call;
  - write methods acquire short-lived write tx per call;
  - single writer is enforced by MDBX and outer `Arc<RwLock<_>>` usage in node wiring.
- Transactions are never stored in struct fields and never cross thread boundaries.

## Key Invariants

- Every write path (`commit`, inserts, genesis) either commits atomically or returns error with no partial durability contract.
- Account/state reads reflect a consistent read-transaction snapshot.
- Empty/missing account returns `None`; missing code returns default bytecode; missing storage returns `U256::ZERO`.
- State root is trie-based, not in-memory hash parity.

## Blocker Resolution Notes

- **BLK-001:** consumed from `state` trait contract; this crate assumes fallible `StateDb` signatures are canonical.
- **BLK-002:** resolved here by pinning trie input coverage and normalization contract.
- **BLK-003:** this crate requires host prerequisites contract (C compiler + clang/bindgen support at build time, writable DB path at runtime); node-level startup policy enforced in `whirlpool-node`.
