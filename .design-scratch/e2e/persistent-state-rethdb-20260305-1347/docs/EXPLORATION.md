# EXPLORATION

## Scope
- Objective: collect and normalize already-produced exploration findings for persistent state backed by reth MDBX, without re-running exploration.
- Iteration target: e2e `persistent-state-rethdb-20260305-1347`.
- Provenance root: `/home/dev/sahara/web3/agent/playground/whirlpool`.

## Architecture Findings
### Layering and ownership
- Observed layering: `state` (interface) -> `state-memory` (HashMap implementation) -> `app-evm` (generic executor) -> `whirlpool-node` (runtime composition wrapper).
- Provenance: `crates/state/src/traits.rs`, `crates/state-memory/src/lib.rs`, `crates/app-evm/src/lib.rs`, `crates/whirlpool-node/src/main.rs`.

### Generic state flow across execution and RPC
- `EvmApplication<DB>` is generic and requires:
  - `DB: StateProvider + Clone + Send + Sync + 'static + revm::Database + Debug`.
- `StateProvider` is a blanket impl for all `T: StateDb`, preserving one trait boundary for all backends.
- RPC types are generic over `S: StateDb`:
  - `EthRpcContext<S: StateDb>`
  - `EthApiHandler<S: StateDb + Send + Sync + 'static>`.
- Provenance: `crates/app-evm/src/lib.rs`, `crates/app-evm/src/traits.rs`, `crates/rpc-eth/src/lib.rs`.

### Node wiring pattern
- Node shares a single DB handle using `Arc<RwLock<TestStateDb>>` between EVM app and RPC context.
- `TestStateDb` currently wraps `InMemoryStateDb` and delegates both `StateDb` and `revm::Database` behavior.
- Provenance: `crates/whirlpool-node/src/main.rs`, `crates/state-memory/src/lib.rs`.

## Type System Findings
### `StateDb` trait surface
- Current contract has 11 infallible methods:
  - `new()`
  - `with_genesis(HashMap<Address, GenesisAccount>)`
  - `state_root() -> B256`
  - `commit(&mut BundleState)`
  - `get_account(Address) -> Option<AccountInfo>`
  - `get_code_by_hash(B256) -> Option<Bytecode>`
  - `get_storage(Address, U256) -> Option<U256>`
  - `get_block_hash(u64) -> Option<B256>`
  - `insert_account(Address, DbAccount)`
  - `insert_block_hash(u64, B256)`
- Type aliases/records:
  - `GenesisAccount = alloy_genesis::GenesisAccount`
  - `DbAccount { info: AccountInfo, storage: HashMap<U256, U256> }`
- Provenance: `crates/state/src/traits.rs`.

### In-memory reference semantics
- `InMemoryStateDb` shape:
  - `accounts: HashMap<Address, DbAccount>`
  - `bytecodes: HashMap<B256, Bytecode>`
  - `block_hashes: HashMap<u64, B256>`
- Trait implementations: `StateDb`, `DatabaseRef`, `Database`, `Clone`, `Debug`.
- `state_root()` is deterministic (sorted material + hash) to avoid map-order nondeterminism.
- Error marker type: `StateError::Internal(String)` + `DBErrorMarker` for revm DB bounds.
- Provenance: `crates/state-memory/src/lib.rs`.

### Cross-crate EVM state types
- Shared state types crossing trait boundary and revm integration:
  - `BundleState`, `Address`, `B256`, `U256`, `AccountInfo`, `Bytecode`.
- Provenance: `crates/state/src/traits.rs`, `crates/state-memory/src/lib.rs`, `crates/app-evm/src/lib.rs`.

### Types addendum (gap closure)
- `StateDb` in `crates/state/src/traits.rs` has no associated types and no lifetime parameters; all methods are concrete and infallible at trait boundary.
- Exact `StateDb` signatures differ from earlier note: `get_code_by_hash -> Bytecode`, `get_storage -> U256`, `get_block_hash -> B256`, `insert_account` takes `AccountInfo` (not `DbAccount`), and method count is 10.
- `InMemoryStateDb` revm adapter methods that `state-reth` must mirror for execution compatibility:
  - `DatabaseRef`: `basic_ref(&self, Address) -> Result<Option<AccountInfo>, StateError>`, `code_by_hash_ref(&self, B256) -> Result<Bytecode, StateError>`, `storage_ref(&self, Address, U256) -> Result<U256, StateError>`, `block_hash_ref(&self, u64) -> Result<B256, StateError>`.
  - `Database`: `basic(&mut self, Address) -> Result<Option<AccountInfo>, StateError>`, `code_by_hash(&mut self, B256) -> Result<Bytecode, StateError>`, `storage(&mut self, Address, U256) -> Result<U256, StateError>`, `block_hash(&mut self, u64) -> Result<B256, StateError>`.
- `StateError` implements `revm::database::DBErrorMarker`; persistent backend error type used in revm `Database`/`DatabaseRef` should satisfy the same marker requirement.
- `StateProvider` blanket impl itself only requires `T: StateDb`; additional backend constraints come from consumers:
  - `EvmApplication<DB>`: `DB: StateProvider + Clone + Send + Sync + 'static + revm::Database + Debug`.
  - `EthApiHandler<S>`: `S: StateDb + Send + Sync + 'static`.
  - `EthRpcContext<S>` stores `Arc<RwLock<S>>`, reinforcing thread-safe shared ownership expectations.
- Provenance: `crates/state/src/traits.rs`, `crates/state-memory/src/db.rs`, `crates/state/src/error.rs`, `crates/app-evm/src/traits.rs`, `crates/app-evm/src/executor.rs`, `crates/rpc-eth/src/context.rs`, `crates/rpc-eth/src/eth_handler.rs`.

## Dependency Findings
### Crate dependency snapshots
- `state`: `revm` 34, `thiserror` 2, `alloy-genesis` 1.5.
- `state-memory`: local `state`, `revm` 34, `sha2` 0.10, `alloy-genesis` 1.5.
- `whirlpool-node`: `app`, `app-evm`, `rpc-eth`, `reth-revm`, `state`, `state-memory`, `revm` 34, `alloy-primitives` 1.5, `tokio` 1, `tracing` 0.1.
- No `[workspace.dependencies]` centralization found.
- Provenance: `crates/state/Cargo.toml`, `crates/state-memory/Cargo.toml`, `crates/whirlpool-node/Cargo.toml`, `Cargo.toml`.

### reth storage stack shape
- `reth-db` default feature set includes MDBX (`features = ["mdbx"]`) and pulls `reth-libmdbx`, `reth-db-api`, `reth-fs-util`, `reth-storage-errors`.
- `reth-db-api` depends on codecs/models/primitives (`reth-codecs`, `reth-db-models`, `reth-ethereum-primitives`, `reth-primitives-traits`).
- `reth-provider` has much larger transitive surface (`reth-db` + trie stacks + provider layers).
- Provenance: `vendor/reth/crates/storage/db/Cargo.toml`, `vendor/reth/crates/storage/db-api/Cargo.toml`, `vendor/reth/crates/storage/provider/Cargo.toml`.

### Dependency gap addendum (conflicts/features/patches)
- `alloy-primitives` compatibility check: no version conflict detected.
  - workspace in-scope crates pin `alloy-primitives = 1.5.0` (`app-evm`, `rpc-eth`, `whirlpool-node`).
  - reth workspace resolves `alloy-primitives = 1.5.0` (`default-features = false`, feature `map-foldhash`) and `reth-db-api` consumes it via `workspace = true`.
- `revm` compatibility check: no version conflict detected.
  - workspace in-scope crates use `revm = 34`.
  - reth workspace pins `revm = 34.0.0` (`default-features = false`), and `reth-revm` forwards revm features; semver line matches.
- Feature interaction note:
  - `reth-db` enables MDBX by default (`default = ["mdbx"]`), which pulls native storage stack automatically unless default features are disabled explicitly.
  - `reth-revm` defaults to `std`; optional checks/witness/memory-limit are feature-gated and additive.
- Workspace patch scan:
  - whirlpool root has no `[workspace.dependencies]` and no `[patch.*]` sections.
  - `vendor/reth/Cargo.toml` contains `[workspace.dependencies]` and `[patch.crates-io]`; patch entries are currently commented (inactive), so no active crate override is applied from vendor reth.
- `reth-mdbx-sys` native build requirements:
  - declared build deps are `cc` and `bindgen`.
  - `build.rs` compiles bundled `libmdbx/mdbx.c` and generates bindings from `mdbx.h`.
  - practical environment requirement: C compiler + libclang/clang available for bindgen; no CMake requirement is declared by crate metadata.
- Provenance: `Cargo.toml`, `crates/app-evm/Cargo.toml`, `crates/rpc-eth/Cargo.toml`, `crates/whirlpool-node/Cargo.toml`, `vendor/reth/Cargo.toml`, `vendor/reth/crates/storage/db/Cargo.toml`, `vendor/reth/crates/storage/db-api/Cargo.toml`, `vendor/reth/crates/revm/Cargo.toml`, `vendor/reth/crates/storage/libmdbx-rs/mdbx-sys/Cargo.toml`, `vendor/reth/crates/storage/libmdbx-rs/mdbx-sys/build.rs`.

## reth-db API Pattern Findings
### Initialization and table setup
- DB creation follows `create_db(path, DatabaseArguments)`; table initialization follows `init_db(path, args)` patterns.
- Provenance: `vendor/reth/crates/storage/db`.

### Table-level mappings for StateDb parity
- Accounts: `PlainAccountState` (`Address -> Account`).
- Storage: `PlainStorageState` dupsort (`Address -> StorageEntry`).
- Bytecode: `Bytecodes` (`B256 -> Bytecode`).
- Block hash and trie tables (`AccountsTrie`, `StoragesTrie`) are relevant for root computation and chain state.
- Provenance: `vendor/reth/crates/storage/db-api`, `vendor/reth/crates/storage/db`, `vendor/reth/crates/trie`.

### Read/write transaction patterns
- Reads:
  - `tx.get_by_encoded_key::<PlainAccountState>(...)`
  - dupsort access via `cursor_dup_read`
  - code fetch from `Bytecodes` by hash.
- Writes:
  - `tx.put::<Table>(key, value)`
  - dupsort mutation via `cursor_dup_write().upsert(...)`
  - durability via `tx.commit()`.
- Provenance: `vendor/reth/crates/storage/db`, `vendor/reth/crates/storage/db-api`.

### State root computation and encoding
- Root path uses `StateRoot::overlay_root(tx, &hashed_state)` over trie-backed data.
- Table payload encoding follows `reth_codecs::Compact` for account/storage/code records.
- Provenance: `vendor/reth/crates/trie`, `vendor/reth/crates/storage/codecs`.

## Design-Relevant Decision Capture
- Raw `reth-db` table access is recommended over `reth-provider` for this migration because it aligns directly with the current 11-method `StateDb` contract and avoids unnecessary provider-level dependency growth.
- Provenance: dependency and API findings in this document (`vendor/reth/crates/storage/*`, `crates/state/src/traits.rs`).

## Post-Processing Results
### Interface audit
- PASS: Existing consumer bounds (`app-evm`, `rpc-eth`) already abstract over `StateDb`/`StateProvider`; backend replacement can be done under current interface.
- Provenance: `crates/state/src/traits.rs`, `crates/app-evm/src/traits.rs`, `crates/rpc-eth/src/lib.rs`.

### Domain auto-split check
- PASS (no additional split required): backend implementation can be introduced as `state-reth` while preserving current interface/consumer boundaries.
- Provenance: `crates/state/src/lib.rs`, `crates/state-memory/src/lib.rs`, `crates/whirlpool-node/src/main.rs`.

### Type disambiguation
- `GenesisAccount` is the alloy genesis input type at the trait boundary.
- `DbAccount` is internal runtime account representation (info + storage map).
- revm `AccountInfo`/`Bytecode` are execution-facing types and must be codec-translated when persisted in reth tables.
- Provenance: `crates/state/src/traits.rs`, `crates/state-memory/src/lib.rs`, `vendor/reth/crates/storage/db-api`, `vendor/reth/crates/storage/codecs`.
