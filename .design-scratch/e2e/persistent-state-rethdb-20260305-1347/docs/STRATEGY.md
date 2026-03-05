# STRATEGY

## Crate Allocation

### New Crate: `state-reth`
**Purpose:** MDBX-backed implementation of `StateDb` trait via raw `reth-db` table access.

**Responsibilities:**
- Implement `StateDb` trait over MDBX persistence
- Implement `revm::DatabaseRef` + `revm::Database` for EVM execution compatibility
- Provide MDBX database initialization and management
- Map `StateDb` methods to reth-db tables (`PlainAccountState`, `PlainStorageState`, `Bytecodes`)
- Handle codec translation between revm types and reth storage models
- Manage state root computation via reth-trie integration

**Label:** [GROUNDED] — derived from INTENT.md requirements and exploration findings on reth-db API patterns.

### Modified Crate: `whirlpool-node`
**Change:** Replace `TestStateDb` + `InMemoryStateDb` wiring with `state-reth` persistent backend.

**Responsibilities:**
- Initialize MDBX database on startup (path configuration)
- Instantiate `state-reth` implementation and wrap in `Arc<RwLock<...>>`
- Wire persistent backend into `EvmApplication` and `EthRpcContext`
- Handle genesis initialization on first run

**Label:** [GROUNDED] — derived from INTENT.md wiring requirements and exploration node composition findings.

### Unchanged Crates
- `state`: Interface authority; trait remains unchanged.
- `state-memory`: Reference implementation for testing and semantics baseline.
- `app-evm`: Generic consumer; already abstracted via `StateProvider` blanket impl.
- `rpc-eth`: Generic consumer; already abstracted via `StateDb` trait bounds.

**Label:** [GROUNDED] — interface audit confirmed no API changes needed for consumers.

---

## Module Boundaries (within `state-reth`)

### Module: `lib.rs`
- Public crate root exposing primary `RethStateDb` type
- Re-exports for initialization helpers and error types

### Module: `db.rs`
- Core `RethStateDb` struct holding MDBX environment/connection handle
- `StateDb` trait implementation (11 methods)
- `revm::DatabaseRef` + `revm::Database` trait implementations
- Transaction lifecycle management (read/write tx acquisition, commit)

### Module: `tables.rs`
- Table access helpers for `PlainAccountState`, `PlainStorageState`, `Bytecodes`
- Key encoding utilities (address/hash to MDBX keys)
- Cursor management for dupsort storage reads/writes

### Module: `codec.rs`
- Type translation layer between revm execution types and reth storage models
- `AccountInfo` <-> reth `Account` codec
- `Bytecode` <-> reth `Bytecode` codec
- `StorageEntry` encoding/decoding

### Module: `trie.rs`
- State root computation via `reth-trie::StateRoot::overlay_root`
- Hashed state preparation from current DB state
- Integration with trie-backed tables (AccountsTrie, StoragesTrie)

### Module: `init.rs`
- Database creation and initialization (`create_db`, `init_db`)
- Genesis account insertion helpers
- Table schema setup

### Module: `error.rs`
- `RethStateError` type unifying MDBX I/O errors and codec failures
- Implements `revm::primitives::DBError` via `DBErrorMarker`
- Conversion from `reth_storage_errors::db::DatabaseError`

**Label:** [PROPOSED] — module factoring based on responsibility clustering from exploration findings.

---

## Trait Design: StateDb Infallibility Bridging

### Current Constraint [GROUNDED]
- `StateDb` trait is infallible (no `Result` returns).
- MDBX operations (read/write/commit) are fallible (return `Result<T, DatabaseError>`).
- `revm::Database` + `revm::DatabaseRef` require `type Error = impl DBError` with fallible methods.

### Selected Strategy [PROPOSED]
**Approach:** **Make `StateDb` trait fallible** with minimal, targeted changes.

**Rationale:**
1. Infallible trait + fallible I/O forces panic-on-error or silent failure — both unacceptable for production persistence.
2. `revm::Database` is already fallible; EVM execution path expects error handling.
3. Consumers (`app-evm`, `rpc-eth`) already handle `revm::Database` errors in execution context.
4. Minimal trait surface adjustment: return `Result<T, Self::Error>` from I/O methods; add associated `type Error`.

**Trait Modifications (state crate):**
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

**Impact:**
- `state-memory::InMemoryStateDb` must add `type Error = Infallible` (or similar never-fails type) and wrap returns in `Ok(...)`.
- `app-evm` execution path already propagates `revm::Database::Error`; adding `StateDb::Error` propagation is natural extension.
- `rpc-eth` handlers will need error mapping from `StateDb::Error` to RPC error responses.

**Alternative Considered (rejected):** Internal retry + panic. Rejected because silent data loss or unexpected panics break production correctness guarantees.

---

## Error Handling Strategy

### Error Type Hierarchy [PROPOSED]
```rust
// state-reth/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum RethStateError {
    #[error("Database error: {0}")]
    Database(#[from] reth_storage_errors::db::DatabaseError),
    
    #[error("Codec error: {0}")]
    Codec(String),
    
    #[error("State root computation failed: {0}")]
    StateRoot(String),
    
    #[error("Initialization error: {0}")]
    Init(String),
}

impl revm::primitives::DBError for RethStateError {}
```

### Error Mapping [PROPOSED]
- MDBX I/O errors (`DatabaseError`) propagate through `StateDb::Error` and `revm::Database::Error` to execution layer.
- Codec translation failures (revm <-> reth types) surface as `RethStateError::Codec`.
- State root computation failures (trie errors) surface as `RethStateError::StateRoot`.
- Consumer layers (`app-evm`, `rpc-eth`) map `StateDb::Error` to domain-specific error responses.

### Recovery Policy [PROPOSED]
- **Read failures:** Propagate to caller (EVM execution or RPC handler decides retry/abort).
- **Write failures:** Abort transaction; rollback via MDBX transaction drop.
- **Commit failures:** Propagate to node layer; node decides restart/recovery strategy.
- **Initialization failures:** Fatal; node startup aborts with error log.

**Label:** Error types grounded in exploration findings (reth error surfaces, revm DBError requirement); recovery policy is proposed design choice.

---

## Concurrency Model

### Current Node Pattern [GROUNDED]
- Node shares state via `Arc<RwLock<S>>` where `S: StateDb`.
- EVM execution acquires write lock during `commit`.
- RPC handlers acquire read locks for queries.
- Consumer bounds require `Send + Sync + 'static`.

### MDBX Transaction Constraints [GROUNDED]
- MDBX read transactions: multiple concurrent readers allowed.
- MDBX write transactions: single writer (exclusive); must be committed or aborted on same thread.
- Transaction handles are not `Send`/`Sync`; environment handle is `Send + Sync`.

### Adapter Strategy [PROPOSED]
**Approach:** **Hold MDBX environment in `RethStateDb`, acquire transactions per method call.**

**Design:**
```rust
pub struct RethStateDb {
    env: Arc<DatabaseEnv>,  // Shared environment (Send + Sync)
    path: PathBuf,
}

impl StateDb for RethStateDb {
    fn get_account(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let tx = self.env.tx_read()?;  // Short-lived read tx
        let account = tx.get::<PlainAccountState>(address)?;
        // tx drops here (auto-abort)
        Ok(account.map(|a| codec::to_account_info(a)))
    }

    fn commit(&mut self, bundle: &BundleState) -> Result<(), Self::Error> {
        let tx = self.env.tx_write()?;  // Write tx
        // Apply all changes
        for (address, account) in bundle.state.iter() {
            tx.put::<PlainAccountState>(address, codec::from_account_info(account))?;
        }
        tx.commit()?;  // Durability
        Ok(())
    }
}
```

**RwLock Interaction:**
- `Arc<RwLock<RethStateDb>>` serializes access at Rust level.
- Read methods hold Rust read lock + acquire MDBX read tx (multiple concurrent reads possible).
- Write methods hold Rust write lock + acquire MDBX write tx (single writer enforced by RwLock).
- MDBX tx lifetime contained within method call (no cross-thread leakage).

**Thread Safety:**
- `DatabaseEnv` is `Send + Sync` (reth guarantees).
- `RethStateDb` is `Send + Sync` (only holds environment + path).
- `Clone` implementation: clone environment `Arc`, share underlying MDBX database.

**Label:** [GROUNDED] on MDBX constraints and node pattern; [PROPOSED] on per-method transaction acquisition strategy.

---

## State Root Strategy

### Current Approach [GROUNDED]
- `state-memory`: deterministic hash over sorted serialized state (keccak256).
- reth approach: trie-based `StateRoot::overlay_root(tx, hashed_state)`.

### Alignment Decision [GROUNDED]
- User-approved at alignment gate: **adopt reth trie semantics** (`StateRoot::overlay_root`), not current keccak256 approach.

### Implementation Path [PROPOSED]
1. Compute hashed state from current DB state (PlainAccountState + PlainStorageState).
2. Invoke `reth_trie::StateRoot::overlay_root` with MDBX read transaction + hashed state.
3. Return computed root as `B256`.

**Code Pattern:**
```rust
fn state_root(&self) -> Result<B256, Self::Error> {
    let tx = self.env.tx_read()?;
    let hashed_state = self.compute_hashed_state(&tx)?;
    let root = StateRoot::overlay_root(&tx, &hashed_state)
        .map_err(|e| RethStateError::StateRoot(e.to_string()))?;
    Ok(root)
}
```

**Dependencies:**
- Requires `reth-trie` crate (already part of reth storage stack).
- Requires trie-backed tables (AccountsTrie, StoragesTrie) initialized via `init_db`.

**Correctness Note [PROPOSED]:**
- Trie root is canonical for Ethereum state; aligns with reth/geth semantics.
- Migration from keccak256 approach means state roots will differ from `state-memory` for same state.
- Test strategy must validate trie root correctness independently (not against in-memory baseline).

**Label:** Root strategy alignment is [GROUNDED]; implementation details are [PROPOSED].

---

## Table Mapping

### StateDb Method -> reth-db Table [GROUNDED]

| `StateDb` Method | reth-db Table | Access Pattern |
|------------------|---------------|----------------|
| `get_account(Address)` | `PlainAccountState` | `tx.get::<PlainAccountState>(address)` |
| `insert_account(Address, AccountInfo)` | `PlainAccountState` | `tx.put::<PlainAccountState>(address, account)` |
| `get_storage(Address, U256)` | `PlainStorageState` (dupsort) | `cursor_dup_read().seek_by_key_subkey(address, index)` |
| `commit` (storage writes) | `PlainStorageState` (dupsort) | `cursor_dup_write().upsert(address, storage_entry)` |
| `get_code_by_hash(B256)` | `Bytecodes` | `tx.get::<Bytecodes>(hash)` |
| `commit` (code writes) | `Bytecodes` | `tx.put::<Bytecodes>(hash, bytecode)` |
| `get_block_hash(u64)` | `CanonicalHeaders` or `HeaderNumbers` | TBD — exploration identified trie tables, block-hash table needs API confirmation |
| `insert_block_hash(u64, B256)` | `CanonicalHeaders` or `HeaderNumbers` | TBD — write pattern needs API confirmation |
| `state_root()` | `PlainAccountState` + `PlainStorageState` + trie tables | Read all state, compute trie root |
| `with_genesis(HashMap)` | `PlainAccountState` + `PlainStorageState` + `Bytecodes` | Batch insert via write tx |

**Label:** Core mappings are [GROUNDED] from exploration; block-hash table detail is [PROPOSED] pending API confirmation.

### Codec Translation [PROPOSED]
- `AccountInfo` (revm) <-> `Account` (reth): map balance/nonce/code_hash fields.
- `Bytecode` (revm) <-> `Bytecode` (reth): likely direct or minimal wrapper.
- `StorageEntry` encoding: `(U256 key, U256 value)` serialized via reth `Compact` codec.

---

## Dependency Plan

### New Dependencies for `state-reth` [PROPOSED]
```toml
[dependencies]
state = { path = "../state" }
reth-db = { path = "../../vendor/reth/crates/storage/db", default-features = false, features = ["mdbx"] }
reth-db-api = { path = "../../vendor/reth/crates/storage/db-api" }
reth-storage-errors = { path = "../../vendor/reth/crates/storage/errors" }
reth-trie = { path = "../../vendor/reth/crates/trie" }
reth-codecs = { path = "../../vendor/reth/crates/storage/codecs" }
revm = "34"
alloy-primitives = "1.5"
thiserror = "2"
```

### Modified Dependencies for `whirlpool-node` [PROPOSED]
```toml
[dependencies]
# ... existing ...
state-reth = { path = "../state-reth" }
```

**Label:** [PROPOSED] — derived from exploration dependency findings and table mapping requirements.

---

## Implementation Phases

### Phase 1: Crate Skeleton + Error Handling
1. Create `crates/state-reth/` directory structure.
2. Implement `RethStateError` with reth error integration.
3. Update `StateDb` trait in `state` crate to add associated `Error` type and `Result` returns.
4. Update `state-memory` to use `type Error = Infallible` and wrap returns.

### Phase 2: Core Table Access
1. Implement `RethStateDb::new` with MDBX `create_db` + `init_db`.
2. Implement read methods: `get_account`, `get_storage`, `get_code_by_hash`, `get_block_hash`.
3. Implement codec layer for type translation.
4. Add unit tests for table access.

### Phase 3: Write Path + Commit
1. Implement `insert_account`, `insert_block_hash`.
2. Implement `commit` with `BundleState` application to tables.
3. Implement write transaction commit/rollback.
4. Add integration tests for write durability.

### Phase 4: State Root + Genesis
1. Implement `state_root` via `reth-trie::StateRoot::overlay_root`.
2. Implement `with_genesis` for initial state setup.
3. Validate trie root correctness against known test vectors.

### Phase 5: revm Integration
1. Implement `revm::DatabaseRef` for read-only EVM access.
2. Implement `revm::Database` for read-write EVM access.
3. Test EVM execution over persistent state.

### Phase 6: Node Wiring
1. Update `whirlpool-node` to initialize `RethStateDb` on startup.
2. Wire persistent backend into `EvmApplication` and `EthRpcContext`.
3. Add configuration for MDBX database path.
4. Handle genesis initialization on first run.
5. Integration test: full node startup with persistent state.

**Label:** [PROPOSED] — phased approach based on dependency ordering and risk mitigation.

---

## Risk Mitigation

### Risk 1: State Root Parity [GROUNDED]
**Mitigation:**
- Validate trie root implementation against Ethereum test vectors (not against in-memory keccak256).
- Document state root semantic change in migration notes.

### Risk 2: MDBX Concurrency [GROUNDED]
**Mitigation:**
- Enforce transaction lifetime discipline: acquire per method, commit/abort before return.
- Add concurrency stress tests: concurrent reads + single writer under `Arc<RwLock<...>>`.

### Risk 3: Trait Fallibility Migration [PROPOSED]
**Mitigation:**
- Update consumers (`app-evm`, `rpc-eth`) to propagate `StateDb::Error` before testing.
- Provide adapter shim if incremental migration is needed.

### Risk 4: Codec Translation Bugs [PROPOSED]
**Mitigation:**
- Property tests: round-trip codec translation (revm -> reth -> revm).
- Validate against state-memory baseline for simple cases before divergence.

---

## Acceptance Criteria

1. `state-reth` crate builds and passes unit tests.
2. `StateDb` trait is fallible; `state-memory` adapted to new signature.
3. `RethStateDb` implements all `StateDb` methods over MDBX tables.
4. `state_root` returns trie-based root via `reth-trie`.
5. `commit` writes durably to MDBX; transaction rollback on error.
6. `revm::Database` + `revm::DatabaseRef` implemented and tested.
7. `whirlpool-node` starts with persistent state; genesis initialization succeeds.
8. Full integration test: EVM execution + RPC queries over persistent backend.
9. Concurrency test: multiple readers + single writer under `Arc<RwLock<...>>`.

**Label:** [PROPOSED] — derived from intent requirements and exploration risk assessment.
