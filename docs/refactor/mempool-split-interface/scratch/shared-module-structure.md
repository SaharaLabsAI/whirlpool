# Shared Module Structure — mempool-split-interface

## mempool Crate
### Module Hierarchy
- `src/lib.rs` declares the public modules `error`, `persistent`, and `store`.
- `src/error.rs` defines the `pub enum MempoolError` plus `Display`, `Error`, and `From` conversions for `std::io::Error` and `reth_libmdbx::Error`.
- `src/store.rs` owns the `pub struct MempoolStore` plus `pub fn open`, `pub fn push`, and `pub fn drain_pending`, wrapping an MDBX `Environment`/`Database` and keeping an `AtomicU64` key counter; `load_next_key` remains private.
- `src/persistent.rs` defines `pub struct PersistentTxPool` (private `store: MempoolStore`) and exposes `pub fn open`, along with the `TxSource` implementation that delegates `push`/`pending` to the store.
  Tests for each module live inside `#[cfg(test)] mod tests`, which stay private to their modules.

### Re-export Chains
- Crate root re-exports the public API surface: `pub use error::MempoolError;`, `pub use persistent::PersistentTxPool;`, and `pub use store::MempoolStore`.
- There are no intermediate `pub use` statements inside `error.rs`, `store.rs`, or `persistent.rs`; the only exports are the structs/enums themselves.

### Visibility Map
- `lib.rs` modules are all `pub mod`, so submodules are accessible to consumers (e.g., `mempool::store`).
- `MempoolError` is `pub enum` with only public variants; its `Display`/`From` impls are public by default.
- `MempoolStore` is a `pub struct` whose methods `open`, `push`, and `drain_pending` are `pub`. The stored MDBX environment, database, and counter fields remain private, and `load_next_key` is a private helper.
- `PersistentTxPool` is `pub struct` with a private `store` field. `open` is `pub`, and the `TxSource` trait implementation exposes `push` and `pending` as required by the trait.
- Module-level `tests` are gated behind `#[cfg(test)]` and are private by default.

### Internal Cross-References
- `store.rs` imports `crate::error::MempoolError` for unified error handling and depends directly on `reth_libmdbx::{Environment, Database, WriteFlags}`.
- `persistent.rs` imports `app::traits::TxSource` to implement the trait, and uses `crate::{MempoolError, MempoolStore}` so it sits one level above the raw store.
- `MempoolError` converts both `std::io::Error` and `reth_libmdbx::Error`, keeping the error plumbing centralized for store/persistent callers.
- `PersistentTxPool` delegates both `push` and `pending`/`drain_pending` to `MempoolStore`, swallowing errors with `eprintln!` so the TxSource trait remains infallible at the call site.

## whirlpool-node (Consumer)
### Import Patterns
- `whirlpool-node/src/main.rs` imports the persistent pool via `use mempool::PersistentTxPool;` only; it never references `MempoolStore` or `MempoolError` directly.
- The node opens the pool with `PersistentTxPool::open(&PathBuf)` and immediately wraps it into an `Arc<dyn TxSource>` (from `app::traits::TxSource`) before handing it to the EVM application and RPC context.
- All calls to the mempool functionality happen through the `TxSource` trait object (`push`/`pending`), so consumers never touch MDBX APIs.

## Reference Pattern: state / state-memory
### state Module Structure
- `crates/state/src/lib.rs` exposes `pub mod block_storage`, `error`, and `traits`, then re-exports the public-facing pieces: `GenesisAccount`, `BlockStorage`, `BlockStorageError`, `StateError`, and `StateDb`.
- `block_storage.rs` defines the `BlockStorage` trait (with methods like `store_block`, `get_block_by_number`, etc.) plus `BlockStorageError`.
- `error.rs` defines the lightweight `StateError` and implements `revm::database::DBErrorMarker`.
- `traits.rs` defines the `StateDb` trait that implementation crates (e.g., `state-memory`) consume.
- The root crate therefore serves as the interface: trait + error, with a minimal dependency graph, exactly mirroring the intended `mempool` interface crate.

### state-memory Module Structure
- `crates/state-memory/src/lib.rs` exposes a single `pub mod db` and immediately re-exports `DbAccount` and `InMemoryStateDb`.
- `db.rs` implements a concrete in-memory `StateDb` and implements `revm::Database`/`DatabaseRef`/`StateDb` traits, depending on `state::traits::StateDb` and `state::error::StateError`.
- The implementation crate carries the heavy dependencies (`revm`, `sha2`, etc.) while consumers rely on the interface crate for trait definitions.

### Applicable Patterns for mempool split
- Mirror the `state` interface crate by letting `mempool` own trait(s) and `MempoolError`, re-exporting only those symbols at the crate root.
- Model the implementation crate after `state-memory`, re-exporting concrete types (`MdbxMempoolStore`, `PersistentTxPool`) while depending on the interface crate for traits/errors.
- Keep public facades minimal (crate root re-exports) so that consumers like `whirlpool-node` can stay pinned to traits/TxSource rather than backend specifics.

## Facade Analysis
- Current facade: `lib.rs` re-exports `MempoolError`, `PersistentTxPool`, and `MempoolStore`. This exposes both interface (error/trait surface) and implementation (MDBX store/pool) from a single crate.
- The split should retain `MempoolError` (and the future `MempoolStore` trait) in the interface crate, while moving `PersistentTxPool` and the MDBX-backed store into `mempool-mdbx`.
- With the interface-focused `mempool` root mirroring `state`, the new facade keeps `app::traits::TxSource` as the public consumer contract and hides implementation details behind the new `mempool-mdbx` crate.
