# Codebase Grounding

## 1. Workspace Cargo.toml
- Path: Cargo.toml
- Members location: lines 4-18 (the `members` array starts at line 4 and ends at 18)
- Current members list: ["crates/consensus", "crates/consensus-simplex", "crates/p2p", "crates/p2p-commonware", "crates/rpc-eth", "crates/whirlpool-node", "crates/state", "crates/state-memory", "crates/state-reth", "crates/app", "crates/app-evm", "crates/mempool", "testing/integration-tests"]

## 2. mempool Cargo.toml
- Path: crates/mempool/Cargo.toml
- app dep: line 7, path = "../app"
- reth-libmdbx dep: line 8, path = "../../vendor/reth/crates/storage/libmdbx-rs"

## 3. mempool lib.rs
- Path: crates/mempool/src/lib.rs
- Mod declarations: lines 1-3 (`pub mod error;`, `pub mod persistent;`, `pub mod store;`).
- Re-exports: lines 5-7 (`pub use error::MempoolError;`, `pub use persistent::PersistentTxPool;`, `pub use store::MempoolStore;`).

## 4. mempool error.rs
- Path: crates/mempool/src/error.rs
- `MempoolError::Mdbx` variant: line 5 (`Mdbx(String)`), `Io` variant on line 6.
- `Display` impl: lines 9-16 with match arms formatting MDBX and IO errors.
- `From<reth_libmdbx::Error>` impl: lines 26-30 converting the error via `to_string()`.

## 5. mempool store.rs
- Path: crates/mempool/src/store.rs
- Use statements: lines 1-8 (std::fs, std::path::Path, std::sync::atomic::{AtomicU64, Ordering}, crate::error::MempoolError, reth_libmdbx::{Database, Environment, WriteFlags}).
- Struct definition: lines 10-14 (`pub struct MempoolStore { ... }` with `env`, `db`, `next_key`).
- Method signatures:
  - `pub fn open(path: &Path)` (lines 16-32).
  - `pub fn push(&self, tx: Vec<u8>)` (lines 34-42).
  - `pub fn drain_pending(&self)` (lines 44-58).
  - `fn load_next_key(env: &Environment, db: &Database)` (lines 60-69).
- Test module boundary: `#[cfg(test)]` on line 73 with tests continuing through line 201 (includes helpers and 9 unit tests verifying push/drain behavior, persistence, FIFO, concurrency).

## 6. mempool persistent.rs
- Path: crates/mempool/src/persistent.rs
- Struct definition: lines 7-9 (`pub struct PersistentTxPool { store: MempoolStore }`).
- `open` constructor: lines 11-15.
- `TxSource` impl: lines 18-34 (`push` writes to store with error logging, `pending` drains store returning stored txs or empty Vec on error).
- Test module boundary: `#[cfg(test)]` on line 36 with tests through line 80 (trait object coercion, pending drains, persistence).

## 7. whirlpool-node Cargo.toml
- Path: crates/whirlpool-node/Cargo.toml
- `mempool` dependency: line 9, specified as `{ path = "../mempool" }`.

## 8. whirlpool-node main.rs
- Path: crates/whirlpool-node/src/main.rs
- Import: line 12 `use mempool::PersistentTxPool;`.
- Usage: lines 113-118 (define `mempool_path`, log opening, `PersistentTxPool::open(&mempool_path)` wrapped in `Arc<dyn TxSource>` and cloned for the EVM application and RPC context).

## 9. mempool integration tests
- Path: crates/mempool/tests/integration.rs
- `use mempool::` imports: line 10 (`use mempool::PersistentTxPool;`).
