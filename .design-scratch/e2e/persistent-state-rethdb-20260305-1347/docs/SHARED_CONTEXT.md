# SHARED CONTEXT

## Workspace Overview
- Workspace root: `/home/dev/sahara/web3/agent/playground/whirlpool`.
- This alignment iteration is focused on persistent state via reth MDBX while preserving the existing `StateDb` interface contract.
- In-scope crates for this pass: `state` (interface), `state-memory` (reference impl), `app-evm` (generic consumer), `rpc-eth` (generic consumer), `whirlpool-node` (wiring), and proposed `state-reth` (new impl crate).

## Architecture Summary (State Flow)
- Layering: `state` (trait) -> `state-memory` (HashMap impl) -> `app-evm` (generic `EvmApplication<DB>`) -> `whirlpool-node` (`TestStateDb` wrapper + runtime wiring).
  - Provenance: `crates/state/src/traits.rs`, `crates/state-memory/src/lib.rs`, `crates/app-evm/src/lib.rs`, `crates/whirlpool-node/src/main.rs`.
- `EvmApplication<DB>` requires `DB: StateProvider + Clone + Send + Sync + 'static + revm::Database + Debug`.
  - Provenance: `crates/app-evm/src/lib.rs`, `crates/app-evm/src/traits.rs`.
- `StateProvider` is a blanket adapter over any `T: StateDb`, preserving backend substitution via trait boundary.
  - Provenance: `crates/app-evm/src/traits.rs`, `crates/state/src/traits.rs`.
- RPC path is similarly generic over `S: StateDb` (`EthRpcContext<S>`, `EthApiHandler<S>`), so backend replacement does not require RPC API redesign.
  - Provenance: `crates/rpc-eth/src/lib.rs`.
- Node composition pattern: `Arc<RwLock<TestStateDb>>` is shared between EVM execution and RPC access; `TestStateDb` delegates to `InMemoryStateDb` for both `StateDb` and `revm::Database` today.
  - Provenance: `crates/whirlpool-node/src/main.rs`, `crates/state-memory/src/lib.rs`.

## Type System Summary
### `StateDb` contract (must remain stable)
- Infallible trait with 10 methods (no associated types, no lifetime parameters):
  - `new()`
  - `with_genesis(HashMap<Address, GenesisAccount>)`
  - `state_root() -> B256`
  - `commit(&mut self, &BundleState)`
  - `get_account(Address) -> Option<AccountInfo>`
  - `get_code_by_hash(B256) -> Bytecode`
  - `get_storage(Address, U256) -> U256`
  - `get_block_hash(u64) -> B256`
  - `insert_account(Address, AccountInfo)`
  - `insert_block_hash(u64, B256)`
- Type aliases/structs at boundary:
  - `GenesisAccount = alloy_genesis::GenesisAccount`
  - `DbAccount` is defined in `state-memory` (`AccountInfo` + storage map), not in `state` trait signature.
- Provenance: `crates/state/src/traits.rs`, `crates/state-memory/src/db.rs`.

### `state-memory` behavioral reference + revm adapter surface
- `InMemoryStateDb` stores three HashMaps: `accounts`, `bytecodes`, `block_hashes`.
- Implements `StateDb`, `revm::DatabaseRef`, `revm::Database`, `Clone`, and `Debug`.
- `state_root()` derives deterministic hashing by sorting serialized state material before hashing.
- `revm::DatabaseRef` impl shape:
  - `type Error = StateError`
  - `basic_ref(&self, Address) -> Result<Option<AccountInfo>, StateError>`
  - `code_by_hash_ref(&self, B256) -> Result<Bytecode, StateError>`
  - `storage_ref(&self, Address, U256) -> Result<U256, StateError>`
  - `block_hash_ref(&self, u64) -> Result<B256, StateError>`
- `revm::Database` impl shape:
  - `type Error = StateError`
  - `basic(&mut self, Address) -> Result<Option<AccountInfo>, StateError>`
  - `code_by_hash(&mut self, B256) -> Result<Bytecode, StateError>`
  - `storage(&mut self, Address, U256) -> Result<U256, StateError>`
  - `block_hash(&mut self, u64) -> Result<B256, StateError>`
- Error type surface includes `StateError::Internal(String)` + `DBErrorMarker` for revm compatibility.
- Provenance: `crates/state-memory/src/db.rs`, `crates/state/src/error.rs`.

### Consumer bounds that constrain persistent backend wrappers
- `StateProvider` blanket impl in `app-evm` only requires `T: StateDb` and forwards `state_root`/`commit`.
- `EvmApplication<DB>` requires `DB: StateProvider + Clone + Send + Sync + 'static + revm::Database + Debug`.
- RPC handler requires `S: StateDb + Send + Sync + 'static`.
- `EthRpcContext<S>` stores `Arc<RwLock<S>>`, reinforcing thread-safe sharing in node wiring.
- Provenance: `crates/app-evm/src/traits.rs`, `crates/app-evm/src/executor.rs`, `crates/rpc-eth/src/context.rs`, `crates/rpc-eth/src/eth_handler.rs`.

### EVM/revm-relevant types
- `BundleState`, `Address`, `B256`, `U256`, `AccountInfo`, and `Bytecode` are the principal shared DB/EVM state types crossing crate boundaries.
- Provenance: `crates/state/src/traits.rs`, `crates/state-memory/src/db.rs`, `crates/app-evm/src/executor.rs`.

## Dependency Summary
### Current dependency posture
- `state`: `revm` 34, `thiserror` 2, `alloy-genesis` 1.5.
  - Provenance: `crates/state/Cargo.toml`.
- `state-memory`: local `state`, `revm` 34, `sha2` 0.10, `alloy-genesis` 1.5.
  - Provenance: `crates/state-memory/Cargo.toml`.
- `whirlpool-node`: `app`, `app-evm`, `rpc-eth`, `state`, `state-memory`, `reth-revm`, `revm` 34, `alloy-primitives` 1.5, `tokio` 1, `tracing` 0.1.
  - Provenance: `crates/whirlpool-node/Cargo.toml`.
- Workspace does not currently centralize versions via `[workspace.dependencies]`.
  - Provenance: `Cargo.toml` (workspace root).

### reth MDBX dependency shape
- `reth-db` defaults to `features = ["mdbx"]` and pulls `reth-libmdbx`, `reth-db-api`, `reth-fs-util`, `reth-storage-errors`.
- `reth-db-api` brings codec/model/primitive dependencies (`reth-codecs`, `reth-db-models`, `reth-ethereum-primitives`, `reth-primitives-traits`).
- `reth-provider` introduces substantially larger transitive scope (`reth-db`, `reth-trie`, `reth-trie-db`, and broader provider stack).
- Provenance: `vendor/reth/crates/storage/db/Cargo.toml`, `vendor/reth/crates/storage/db-api/Cargo.toml`, `vendor/reth/crates/storage/provider/Cargo.toml`.

### Gap-focused compatibility and build checks
- `alloy-primitives` alignment: in-scope workspace crates use `alloy-primitives = 1.5.0`, and reth storage crates resolve `alloy-primitives` from reth workspace as `1.5.0` (with `default-features = false`, `features = ["map-foldhash"]`); no version skew detected.
- `revm` alignment: in-scope workspace crates use `revm = 34`, and reth workspace pins `revm = 34.0.0` (`default-features = false`); version line is compatible (feature set differs by crate but unifies without semver conflict).
- Feature interaction note: `reth-db` enables MDBX by default (`default = ["mdbx"]`), while `reth-revm` defaults to `std`; consuming crates should keep this explicit when minimizing feature surface for a new `state-reth` crate.
- Workspace patch/dependency posture:
  - whirlpool root has no `[workspace.dependencies]` and no `[patch.*]`.
  - `vendor/reth/Cargo.toml` has a large `[workspace.dependencies]` catalog and a `[patch.crates-io]` section, but current patch entries are commented (inactive).
- `reth-mdbx-sys` toolchain requirements: build uses `cc` + `bindgen` in `build.rs`, compiling bundled `libmdbx/mdbx.c`; practical host requirements are a C compiler and libclang/clang for bindgen. No CMake requirement is declared in crate build metadata.
- Provenance: `Cargo.toml`, `crates/app-evm/Cargo.toml`, `crates/rpc-eth/Cargo.toml`, `crates/whirlpool-node/Cargo.toml`, `vendor/reth/Cargo.toml`, `vendor/reth/crates/storage/db/Cargo.toml`, `vendor/reth/crates/storage/db-api/Cargo.toml`, `vendor/reth/crates/storage/libmdbx-rs/mdbx-sys/Cargo.toml`, `vendor/reth/crates/storage/libmdbx-rs/mdbx-sys/build.rs`.

## reth-db API Pattern Summary
- DB init pattern: `create_db(path, DatabaseArguments)` returning env/handle; `init_db(path, args)` initializes expected tables.
- Core table mappings relevant to `StateDb` parity:
  - accounts: `PlainAccountState` (`Address -> Account`)
  - storage: `PlainStorageState` dupsort (`Address -> StorageEntry`)
  - code: `Bytecodes` (`B256 -> Bytecode`)
  - block hash/trie-support tables as needed for root derivation.
- Read patterns:
  - `tx.get_by_encoded_key::<PlainAccountState>(...)`
  - dupsort reads via `cursor_dup_read`
  - direct code fetch via `Bytecodes`.
- Write patterns:
  - `tx.put::<Table>(key, value)`
  - dupsort upsert via `cursor_dup_write().upsert(...)`
  - finalize with `tx.commit()`.
- State root pattern: `StateRoot::overlay_root(tx, &hashed_state)` over trie-backed tables.
- Encoding contract uses `reth_codecs::Compact` across account/storage/code payloads.
- Provenance: `vendor/reth/crates/storage/db`, `vendor/reth/crates/trie`, `vendor/reth/crates/storage/codecs`.

## Decision Note
- For this iteration, raw `reth-db` table access is the preferred implementation basis for a new `state-reth` crate because it maps cleanly onto the existing 11-method `StateDb` trait with lower dependency and abstraction overhead than `reth-provider`.
- Provenance: dependency and API pattern findings listed above.

## Post-Processing Checks
### Interface audit
- Result: PASS.
- `StateDb` contract can remain unchanged; existing generic consumers (`app-evm`, `rpc-eth`) already abstract over `StateDb`/`StateProvider` and do not require backend-specific API changes.
- Provenance: `crates/state/src/traits.rs`, `crates/app-evm/src/traits.rs`, `crates/rpc-eth/src/lib.rs`.

### Domain auto-split check
- Result: NO SPLIT REQUIRED for interface domain.
- Storage backend implementation can be isolated to a dedicated `state-reth` crate while preserving current domain boundaries (`state` trait + consumer crates).
- Provenance: `crates/state/src/lib.rs`, `crates/state-memory/src/lib.rs`, `crates/whirlpool-node/src/main.rs`.

### Type disambiguation
- Result: COMPLETE.
- `GenesisAccount` refers to `alloy_genesis::GenesisAccount` (trait boundary input type).
- `DbAccount` is local state representation (`AccountInfo` + storage map), distinct from reth account table encodings.
- `AccountInfo`/`Bytecode` are revm-level execution types; reth persisted models require codec translation at boundary.
- Provenance: `crates/state/src/traits.rs`, `crates/state-memory/src/lib.rs`, `vendor/reth/crates/storage/db-api`, `vendor/reth/crates/storage/codecs`.
