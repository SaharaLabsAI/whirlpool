# state

<!-- continuation round 2: resolves B-002 -->

## Purpose

In-memory EVM state database crate. Provides `InMemoryStateDb` implementing `revm::Database` + `Clone`, state commitment via `BundleState` application, and state root computation. This crate resolves the B-002 blocker by giving `app-evm::EvmApplication<DB>` a concrete `DB` type.

The state crate is a standalone provider — it does not wrap an abstract trait crate (unlike `consensus-simplex` wrapping `consensus`). It directly implements `revm::Database` against in-memory HashMaps.

## Public API at a glance (crate root exports)

[PROPOSED] — all items below are proposed; this crate does not yet exist.

```rust
// lib.rs
pub mod db;
pub mod error;

pub use db::InMemoryStateDb;
pub use error::StateError;
```

## Modules

| Module | Responsibilities |
|---|---|
| `db` | `InMemoryStateDb` struct — `revm::Database` + `DatabaseRef` impl, `commit()`, `state_root()`, genesis initialization |
| `error` | `StateError` — error type for state operations |

## Types & traits (public contract)

### InMemoryStateDb [PROPOSED]

```rust
/// In-memory EVM state database.
/// Stores all account state, contract bytecode, storage slots, and block hashes
/// in HashMaps. Implements revm::Database for direct use by the EVM execution engine.
///
/// Clone produces an independent snapshot — mutations to the clone do not affect
/// the original. This satisfies the `DB: Database + Clone` bound on EvmApplication.
#[derive(Clone, Debug)]
pub struct InMemoryStateDb {
    /// Account state: balance, nonce, code_hash, and per-account storage
    accounts: HashMap<Address, DbAccount>,
    /// Contract bytecodes keyed by code hash
    bytecodes: HashMap<B256, Bytecode>,
    /// Recent block hashes (block number → hash), used for BLOCKHASH opcode
    block_hashes: HashMap<u64, B256>,
}

/// Per-account state
#[derive(Clone, Debug, Default)]
pub struct DbAccount {
    pub info: AccountInfo,
    pub storage: HashMap<U256, U256>,
}
```

**Key trait implementations:**

```rust
impl Database for InMemoryStateDb {
    type Error = StateError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, StateError> {
        // Returns AccountInfo from accounts map, or Ok(None) for unknown addresses
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, StateError> {
        // Returns bytecode from bytecodes map, or Ok(Bytecode::default()) for missing
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, StateError> {
        // Returns storage value, or Ok(U256::ZERO) for missing slots/accounts
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, StateError> {
        // Returns block hash, or Ok(B256::ZERO) for missing entries
    }
}

impl DatabaseRef for InMemoryStateDb {
    type Error = StateError;
    // Same semantics as Database but via &self (shared reference)
}
```

Construction and state management:

```rust
impl InMemoryStateDb {
    /// Create an empty state database
    pub fn new() -> Self { ... }

    /// Create state from genesis allocation.
    /// `GenesisAccount` is re-exported from `alloy_genesis::GenesisAccount`
    /// (provides `balance: U256`, `nonce: Option<u64>`, `code: Option<Bytes>`,
    /// `storage: Option<HashMap<B256, B256>>`).
    /// BLOCKER: depends on ChainSpec resolution (B-001) for genesis definition.
    pub fn with_genesis(alloc: HashMap<Address, alloy_genesis::GenesisAccount>) -> Self { ... }

    /// Apply execution diff to state. Called after block execution succeeds.
    /// Iterates BundleState.state (changed accounts) and BundleState.contracts
    /// (new bytecodes) and updates internal HashMaps.
    pub fn commit(&mut self, bundle: &BundleState) { ... }

    /// Compute deterministic state root over all accounts and storage.
    /// MVP: flat keccak256 hash over sorted (address, account_info, sorted_storage).
    /// [BLOCKER]: Production requires Merkle Patricia Trie for proof generation.
    pub fn state_root(&self) -> B256 { ... }

    /// Insert a block hash entry (for BLOCKHASH opcode support).
    /// Called after each block is committed to maintain the last 256 block hashes.
    pub fn insert_block_hash(&mut self, number: u64, hash: B256) { ... }
}
```

### StateError [PROPOSED]

```rust
/// Errors from state database operations.
/// For the in-memory implementation, most operations are infallible.
/// This type exists to satisfy revm::Database::Error and to support
/// future persistent backends that may have real I/O errors.
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    /// Generic internal error (future: I/O, serialization)
    #[error("state error: {0}")]
    Internal(String),
}
```

## Config schema

No configuration — `InMemoryStateDb` is constructed programmatically via `new()` or `with_genesis()`.

## Config defaults table

| Field | Type | Default | Source | Evidence |
|---|---|---|---|---|
| Initial accounts | `HashMap<Address, DbAccount>` | empty | `InMemoryStateDb::new()` | [PROPOSED] |
| Genesis accounts | `HashMap<Address, GenesisAccount>` | BLOCKER — depends on B-001 | `InMemoryStateDb::with_genesis()` | [PROPOSED] |

## Provider interfaces & swap points

| Interface | Trait | Default impl | Swap point |
|---|---|---|---|
| EVM state database | `revm::Database` | `InMemoryStateDb` [PROPOSED] | Replace `InMemoryStateDb` with `RocksDbStateDb` (or similar) for persistence. `EvmApplication<DB>` generic bound allows any `Database + Clone` impl. |
| State root algorithm | (internal to `state_root()`) | Flat keccak256 hash [PROPOSED] | Swap to MPT/Verkle trie implementation. Method signature unchanged. |

## Feature flags & cfg

[PROPOSED]:
- `std` (default): full std features
- `test-utils`: builders for test state (pre-funded accounts, deployed contracts)

## SemVer & stability

UNKNOWN — pre-1.0 workspace.

## Primary flows

### 1. Genesis initialization [PROPOSED]
```pseudo
InMemoryStateDb::with_genesis(alloc)
  1. Create empty InMemoryStateDb
  2. For each (address, genesis_account) in alloc:
     a. Create DbAccount { info: AccountInfo { balance, nonce, code_hash }, storage }
     b. Insert into accounts map
     c. If genesis_account has code: insert into bytecodes map
  3. Return populated InMemoryStateDb
```

### 2. Block execution state flow [PROPOSED]
```pseudo
// Called by EvmApplication during propose() or verify()
// EvmApplication holds Arc<RwLock<InMemoryStateDb>> for interior mutability.
// Application trait methods take &self, so we use read lock + clone for snapshots.
self.state_db.read().unwrap().clone()  →  independent snapshot for speculative execution
  ↓
State::new(snapshot.clone())  →  revm State wrapper (accumulates changes)
  ↓
executor.execute_one(...)  →  reads from Database, accumulates changes in State
  ↓
executor.finish()  →  returns BlockExecutionOutput { state: BundleState, result, .. }
  ↓
snapshot.commit(&bundle_state)  →  applies diff to SNAPSHOT (not canonical state)
  ↓
snapshot.state_root()  →  computes new root for block header
  ↓
// Return (block, bundle_state) to caller. Canonical commit happens ONLY on finalization:
// EvmFinalizationSink::finalized() calls:
//   self.state_db.write().unwrap().commit(&bundle_state)
//   self.state_db.write().unwrap().insert_block_hash(height, hash)
```

### 3. Commit processing [PROPOSED]
```pseudo
InMemoryStateDb::commit(bundle_state)
  1. For each (address, account_change) in bundle_state.state:
     a. Match account_change.status:
        - Created → insert new DbAccount with info + initial storage
        - Changed → update existing DbAccount.info, apply storage diff
        - Destroyed → remove entry from accounts map
     b. If storage was wiped: clear all existing storage slots
     c. For each (key, value) in account_change.storage:
        - If value.present == U256::ZERO: remove slot
        - Else: insert/update slot
  2. For each (code_hash, bytecode) in bundle_state.contracts:
     - Insert into bytecodes map
```

### 4. State root computation [PROPOSED]
```pseudo
InMemoryStateDb::state_root()
  1. Collect all accounts, sort by Address (lexicographic)
  2. For each account (sorted):
     a. Sort storage entries by key
     b. RLP-encode or canonically serialize: (address, nonce, balance, code_hash, [(key, value)...])
  3. Iteratively hash all serialized accounts: keccak256(accumulator ++ account_bytes)
  4. Return final B256 hash
  NOTE: NOT a Merkle Patricia Trie — production requires real trie for proof support
```

## API omissions report

- **Persistent storage backend**: `InMemoryStateDb` loses all state on process restart. Production requires a persistent backend (RocksDB, MDBX, or similar). The `Database + Clone` generic bound on `EvmApplication` allows swapping implementations without changing `app-evm`.
- **Merkle Patricia Trie**: State root computation uses a flat hash, not a real trie. This means no support for: state proofs, light client verification, or Ethereum-compatible `eth_getProof` RPC. [BLOCKER for production]
- **Historical state access**: No `StateProviderFactory` pattern — cannot query state at arbitrary block heights. Only current (latest) state is accessible. Needed for `eth_call` at historical blocks.
- **State pruning**: No mechanism to prune old state or limit `block_hashes` to 256 entries. Must be added for long-running nodes.

## Open questions / TODOs

- BLOCKER: Genesis allocation depends on ChainSpec (B-001) — `with_genesis()` cannot be fully defined until chain config is resolved.
- BLOCKER: State root algorithm is a placeholder flat hash. Production requires MPT or Verkle trie with proof support.
- UNKNOWN: Whether `commit()` should take `&BundleState` or `BundleState` (owned). Owned allows consuming the bundle; borrowed allows re-use.
- UNKNOWN: Thread safety — should `InMemoryStateDb` be `Send + Sync`? HashMap-based impl is `Send + Sync` by default, but `Clone` for concurrent reads may need `Arc<RwLock<...>>` for shared access patterns.
- UNKNOWN: Memory budget — no limit on state size. Large state will consume unbounded memory. Consider LRU eviction for `block_hashes` and state snapshots.

<!-- continuation round 2: Arc<RwLock> usage pattern -->

## Shared ownership pattern

When used with `EvmApplication`, `InMemoryStateDb` is wrapped in `Arc<RwLock<InMemoryStateDb>>` to provide interior mutability. The `Application` trait methods (`propose`, `verify`) take `&self`, so direct mutation is not possible. The pattern is:

1. **Read path** (propose/verify): `self.state_db.read().unwrap().clone()` → independent snapshot. Execute against snapshot, compute state root on snapshot. Return `(block, BundleState)`. Do NOT write to canonical state.
2. **Write path** (finalization only): `self.state_db.write().unwrap().commit(&bundle_state)` → applies diff to canonical state. Only called by `EvmFinalizationSink::finalized()` when consensus confirms a block.
3. **Fork safety**: If a proposed block is not finalized (e.g., fork), the snapshot is simply dropped. Canonical state is never corrupted by speculative execution.

This ensures that concurrent reads during verification do not block, and only finalized blocks mutate canonical state. [PROPOSED]
