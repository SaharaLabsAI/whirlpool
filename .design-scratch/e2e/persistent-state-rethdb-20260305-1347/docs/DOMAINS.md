# DOMAINS

## Domains

### Domain 1: Persistent Storage
**Bounded Context:** Low-level MDBX database lifecycle and table access abstraction.

**Purpose:** Encapsulates MDBX environment initialization, transaction management, and direct table read/write operations for accounts, storage, bytecodes, and block-hash mappings.

**Owning Crates:**
- `state-reth` (modules: `init.rs`, `db.rs`, `tables.rs`)

**Key Types and Traits:**
- `RethStateDb` — core struct holding `Arc<DatabaseEnv>` and database path
- `reth_db::DatabaseEnv` — MDBX environment handle (Send + Sync)
- `reth_db_api::transaction::DbTx` / `DbTxMut` — read/write transaction handles
- `reth_db_api::table::Table` trait — typed table access
- reth-db tables: `PlainAccountState`, `PlainStorageState`, `Bytecodes`, `CanonicalHeaders`/`HeaderNumbers` (TBD per BLK-101)

**Public Contract:**
- `create_db(path: PathBuf) -> Result<DatabaseEnv, RethStateError>`
- `init_db(env: &DatabaseEnv) -> Result<(), RethStateError>`
- Per-method transaction acquisition (short-lived read/write tx)
- Write transactions commit before returning (durability guarantee)
- Transaction handles are NOT `Send`/`Sync`; environment handle IS `Send + Sync`

**Dependencies on Other Domains:**
- None (lowest layer — provides storage primitives consumed by State Interface and State Root domains)

**Evidence:**
- `STRATEGY.md` lines 40-76 (module factoring)
- `STRATEGY.md` lines 176-218 (concurrency model + transaction strategy)
- `STRATEGY.md` lines 259-276 (table mapping)
- `CRATES.md` lines 21-89 (state-reth crate allocation)

---

### Domain 2: State Interface
**Bounded Context:** Backend-agnostic state access contract exposed to EVM execution and RPC layers.

**Purpose:** Define the canonical `StateDb` trait contract with fallible operations, and provide codec translation between revm execution types and reth storage models.

**Owning Crates:**
- `state` (trait definition: `StateDb`)
- `state-reth` (implementation modules: `db.rs`, `codec.rs`, `error.rs`)
- `state-memory` (reference implementation with infallible error type)

**Key Types and Traits:**
- `state::StateDb` trait — 11 methods for account/storage/code/block-hash access + state root + commit
- `RethStateError` — unified error type (`Database`, `Codec`, `StateRoot`, `Init` variants)
- `revm::Database` + `revm::DatabaseRef` traits — EVM execution integration
- Codec types: `revm::primitives::AccountInfo` <-> `reth_primitives::Account`, `revm::Bytecode` <-> `reth_primitives::Bytecode`

**Public Contract (Post-BLK-001 Resolution):**
```rust
pub trait StateDb {
    type Error: std::error::Error + Send + Sync + 'static;
    fn new() -> Result<Self, Self::Error> where Self: Sized;
    fn with_genesis(genesis: HashMap<Address, GenesisAccount>) -> Result<Self, Self::Error> where Self: Sized;
    fn state_root(&self) -> Result<B256, Self::Error>;
    fn commit(&mut self, bundle: &BundleState) -> Result<(), Self::Error>;
    fn get_account(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;
    fn get_code_by_hash(&self, hash: B256) -> Result<Bytecode, Self::Error>;
    fn get_storage(&self, address: Address, index: U256) -> Result<U256, Self::Error>;
    fn get_block_hash(&self, number: u64) -> Result<B256, Self::Error>;
    fn insert_account(&mut self, address: Address, info: AccountInfo) -> Result<(), Self::Error>;
    fn insert_block_hash(&mut self, number: u64, hash: B256) -> Result<(), Self::Error>;
}
```

**Dependencies on Other Domains:**
- Depends on **Persistent Storage** for MDBX table access (via `tables.rs` helpers)
- Consumed by **Node Wiring** domain (EVM execution + RPC contexts)

**Evidence:**
- `STRATEGY.md` lines 81-121 (trait design + fallibility bridging)
- `STRATEGY.md` lines 123-160 (error handling strategy)
- `CRATES.md` lines 92-137 (state crate modifications)
- `BLOCKERS.md` BLK-001 (fallibility contract blocker)

---

### Domain 3: State Root
**Bounded Context:** Trie-based state root computation using reth's canonical trie semantics.

**Purpose:** Compute Ethereum-compatible state root via `reth-trie::StateRoot::overlay_root`, replacing in-memory keccak256 approach with trie-backed Merkle root.

**Owning Crates:**
- `state-reth` (module: `trie.rs`)

**Key Types and Traits:**
- `reth_trie::StateRoot` — state root computation engine
- `reth_trie_common::HashedPostState` — hashed state input for overlay root
- Trie tables: `AccountsTrie`, `StoragesTrie` (initialized via `init_db`)

**Public Contract:**
```rust
fn state_root(&self) -> Result<B256, Self::Error> {
    let tx = self.env.tx_read()?;
    let hashed_state = self.compute_hashed_state(&tx)?;
    let root = StateRoot::overlay_root(&tx, &hashed_state)
        .map_err(|e| RethStateError::StateRoot(e.to_string()))?;
    Ok(root)
}
```

**Dependencies on Other Domains:**
- Depends on **Persistent Storage** for read transaction acquisition and trie table access
- Consumed by **State Interface** (implements `StateDb::state_root()`)

**Evidence:**
- `STRATEGY.md` lines 220-256 (state root strategy)
- `CRATES.md` lines 62-63 (`trie.rs` module allocation)
- `BLOCKERS.md` BLK-002 (trie root input contract blocker)

**Known Gaps:**
- [BLOCKER: BLK-002] Exact hashed-state construction contract (source tables, normalization rules) is not yet pinned; this blocks precise correctness validation criteria.

---

### Domain 4: Node Wiring
**Bounded Context:** Runtime composition and lifecycle management for persistent state backend integration into EVM execution and RPC serving.

**Purpose:** Initialize MDBX database on startup, instantiate `state-reth` backend, wire into `EvmApplication` and `EthRpcContext` via `Arc<RwLock<...>>`, and handle genesis bootstrapping on first run.

**Owning Crates:**
- `whirlpool-node` (binary crate: `main.rs` + runtime/config modules)

**Key Types and Traits:**
- `Arc<RwLock<RethStateDb>>` — shared mutable state container
- `EvmApplication<DB>` — EVM execution context (generic over `StateProvider`)
- `EthRpcContext<S>` — RPC handler context (generic over `StateDb`)
- Configuration: MDBX database path, genesis initialization flag

**Public Contract:**
- Node startup sequence:
  1. Parse configuration (database path)
  2. Initialize/open MDBX database (`create_db` + `init_db` or `open`)
  3. Run genesis initialization if first startup (`with_genesis`)
  4. Wrap backend in `Arc<RwLock<...>>`
  5. Wire into EVM app and RPC context constructors
  6. Start runtime servers
- Failure policy: abort startup on initialization errors (fatal)

**Dependencies on Other Domains:**
- Depends on **State Interface** (constructs concrete `RethStateDb` and passes via trait boundary to consumers)
- Depends on **Persistent Storage** indirectly (via `state-reth` initialization helpers)

**Evidence:**
- `STRATEGY.md` lines 18-27 (whirlpool-node change allocation)
- `CRATES.md` lines 140-187 (whirlpool-node crate modifications)
- `BLOCKERS.md` BLK-003 (MDBX host prerequisites blocker)

**Known Gaps:**
- [BLOCKER: BLK-003] MDBX host prerequisites contract (build/runtime requirements, platform assumptions, failure policy) is not yet specified; this blocks operational enablement.

---

## Wiring

| Capability | Owning Crate | Trait Interface | Provider | Config | Evidence |
|---|---|---|---|---|---|
| MDBX environment initialization | `state-reth` | `init::create_db`, `init::init_db` | `reth-db::DatabaseEnv::open` + table schema setup | Database path (`PathBuf`) | STRATEGY.md lines 40-76, CRATES.md lines 62-63 |
| Transaction lifecycle management | `state-reth` | `db.rs` per-method tx acquisition | `DatabaseEnv::tx_read()`, `DatabaseEnv::tx_write()` | None (per-method acquisition policy) | STRATEGY.md lines 176-218 |
| Table access (accounts/storage/code) | `state-reth` | `tables.rs` helpers | `DbTx::get`, `DbTxMut::put`, `cursor_dup_read/write` | Table names: `PlainAccountState`, `PlainStorageState`, `Bytecodes` | STRATEGY.md lines 259-276 |
| Codec translation (revm <-> reth) | `state-reth` | `codec.rs` conversion functions | Manual field mapping | None | STRATEGY.md lines 56-61, STRATEGY.md lines 278-282 |
| State root computation | `state-reth` | `trie.rs::state_root` | `reth_trie::StateRoot::overlay_root` | Trie tables: `AccountsTrie`, `StoragesTrie` | STRATEGY.md lines 220-256, BLOCKERS.md BLK-002 |
| StateDb trait implementation | `state-reth` | `state::StateDb` | `RethStateDb` struct | Associated `type Error = RethStateError` | STRATEGY.md lines 81-121, CRATES.md lines 66-70 |
| revm Database traits | `state-reth` | `revm::Database`, `revm::DatabaseRef` | `RethStateDb` | Error type implements `revm::DBError` | STRATEGY.md lines 86-87, lines 143-145 |
| Genesis initialization | `state-reth` | `StateDb::with_genesis` | `init.rs` batch insert helpers | `HashMap<Address, GenesisAccount>` | STRATEGY.md lines 68-70, INTENT.md line 11 |
| Node runtime wiring | `whirlpool-node` | `main.rs` startup sequence | Constructs `RethStateDb`, wraps in `Arc<RwLock<...>>` | Database path config, genesis flag | STRATEGY.md lines 18-27, CRATES.md lines 176-183 |
| EVM execution state access | `app-evm` | Generic `StateProvider` blanket impl | Receives `Arc<RwLock<impl StateDb>>` | None (generic trait bound) | INTENT.md line 12, CRATES.md line 32 |
| RPC state access | `rpc-eth` | Generic `StateDb` trait bound | Receives `Arc<RwLock<impl StateDb>>` | None (generic trait bound) | INTENT.md line 12, CRATES.md line 33 |

---

## Cross-Domain Boundaries

### Persistent Storage → State Interface
**Direction:** Storage provides table access primitives; State Interface consumes them to implement `StateDb` trait.

**Data Flow:**
- `StateDb::get_account` → `tables::get_account` → `tx.get::<PlainAccountState>` → codec translation → return `AccountInfo`
- `StateDb::commit` → `tables::put_account` + `tables::put_storage` → `tx.put` + `tx.commit` → durability

**Dependency Type:** Implementation dependency (State Interface depends on Storage for MDBX I/O)

**Risks:**
- Transaction lifetime discipline violation (leaking tx handles across method boundaries) → mitigated by per-method acquisition policy
- Codec translation bugs (revm <-> reth type mismatches) → mitigated by property tests (risk BLK-102 soft-blocks taxonomy finalization)

**Evidence:** STRATEGY.md lines 176-218, lines 259-276

---

### State Root → Persistent Storage
**Direction:** State Root reads trie tables and current state tables from Storage layer.

**Data Flow:**
- `StateDb::state_root` → `trie::compute_hashed_state` → read `PlainAccountState` + `PlainStorageState` via read tx
- Invoke `StateRoot::overlay_root(tx, hashed_state)` → read `AccountsTrie` + `StoragesTrie` → return Merkle root

**Dependency Type:** Read-only data dependency (State Root reads from Storage but does not modify)

**Risks:**
- [BLOCKER: BLK-002] Hashed-state input contract not pinned; incorrect normalization could produce wrong roots
- Performance: full state scan for root computation → soft-blocked by BLK-103 (caching/batching optimization deferred)

**Evidence:** STRATEGY.md lines 220-256, BLOCKERS.md BLK-002

---

### Node Wiring → State Interface + Persistent Storage
**Direction:** Node Wiring constructs Storage backend (via State Interface constructors) and passes to consumers.

**Data Flow:**
1. Node startup: `create_db(path)` → `init_db(env)` → `RethStateDb::with_genesis(...)` → persistent backend instance
2. Wrap in `Arc<RwLock<RethStateDb>>`
3. Pass to `EvmApplication<DB>` constructor (expects `impl StateDb`)
4. Pass to `EthRpcContext<S>` constructor (expects `impl StateDb`)

**Dependency Type:** Composition dependency (Node assembles concrete types and injects via trait boundaries)

**Risks:**
- [BLOCKER: BLK-003] MDBX host prerequisites missing → startup failure
- Genesis initialization on first run: race condition if multiple processes attempt simultaneous init → requires filesystem locking or startup serialization
- `Arc<RwLock<...>>` contention under high RPC load → mitigated by short-lived MDBX tx acquisition (storage layer serializes I/O, not Rust-level lock)

**Evidence:** STRATEGY.md lines 18-27, CRATES.md lines 176-183, BLOCKERS.md BLK-003

---

### State Interface → EVM Execution (app-evm)
**Direction:** EVM execution consumes `StateDb` via generic trait boundary.

**Data Flow:**
- EVM opcode execution → `revm::Database::basic(address)` → `StateDb::get_account` → fallible read
- Block execution commit → `revm::Database::commit(bundle)` → `StateDb::commit` → fallible write + MDBX durability

**Dependency Type:** Consumer dependency (app-evm depends on State Interface; State Interface does not depend on app-evm)

**Risks:**
- [BLOCKER: BLK-001] Fallibility migration impact: app-evm must propagate `StateDb::Error` through execution stack
- Error mapping from `RethStateError` to EVM execution errors → requires domain-specific error response design

**Evidence:** STRATEGY.md lines 114-119, CRATES.md line 32, BLOCKERS.md BLK-001

---

### State Interface → RPC Serving (rpc-eth)
**Direction:** RPC handlers consume `StateDb` via generic trait boundary.

**Data Flow:**
- RPC method `eth_getBalance(address)` → `StateDb::get_account` → fallible read
- RPC method `eth_getStorageAt(address, index)` → `StateDb::get_storage` → fallible read

**Dependency Type:** Consumer dependency (rpc-eth depends on State Interface; State Interface does not depend on rpc-eth)

**Risks:**
- [BLOCKER: BLK-001] Fallibility migration impact: rpc-eth must map `StateDb::Error` to JSON-RPC error responses
- Concurrent read contention under high RPC load → mitigated by MDBX concurrent read transactions (multiple readers allowed)

**Evidence:** STRATEGY.md lines 114-119, CRATES.md line 33, BLOCKERS.md BLK-001

---

## Wiring Risks Summary

### Tight Coupling Risks
- **Codec layer (`state-reth::codec.rs`)**: tightly coupled to both revm and reth type surfaces; upstream changes in either require codec updates. Mitigated by limiting codec scope to `AccountInfo`/`Bytecode`/`StorageEntry` conversions only.
- **Trie root semantics divergence**: `state-memory` uses keccak256; `state-reth` uses trie root. Tests written against in-memory baseline will fail if they expect identical roots. Mitigated by explicit documentation and separate test fixtures for trie-backed correctness.

### Circular Dependency Risks
- **None identified.** Dependency graph is acyclic:
  - Storage (lowest layer, no workspace deps)
  - State Interface + State Root (depend on Storage)
  - Node Wiring (depends on State Interface, constructs Storage backend)
  - Consumers (app-evm, rpc-eth) depend only on State Interface trait

### Concurrency Risks
- **RwLock contention**: `Arc<RwLock<RethStateDb>>` serializes access at Rust level, but MDBX allows concurrent reads. Per-method tx acquisition minimizes lock hold time.
- **Write transaction exclusivity**: MDBX enforces single writer; Rust `RwLock` write guard enforces same. No additional serialization needed.
- **Transaction handle thread-safety**: MDBX tx handles are NOT `Send`/`Sync`; mitigated by scoping tx lifetime to single method call (no cross-thread leakage).

**Evidence:** STRATEGY.md lines 163-218, CRATES.md lines 62-64
