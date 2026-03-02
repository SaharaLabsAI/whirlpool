# state

## Purpose

The `state` crate provides the canonical in-memory storage for the EVM state. It is the sole owner of the state management domain, responsible for holding account information, storage slots, bytecode, and block hashes. It derives the deterministic state root used for block verification and provides the `revm::Database` / `DatabaseRef` interface for EVM execution. <!-- GROUNDED -->

## Public API at a glance (crate root exports)
- `DbAccount`: Container for account info and storage. <!-- GROUNDED -->
- `InMemoryStateDb`: Main in-memory database implementation. <!-- GROUNDED -->
- `StateError`: Error type for database operations. <!-- GROUNDED -->

## Modules
- `db`: Core database structures and logic. <!-- GROUNDED -->
- `error`: Error definitions and trait implementations. <!-- GROUNDED -->

## Types & traits (public contract)
- `DbAccount`: `{ info: AccountInfo, storage: HashMap<U256, U256> }` <!-- GROUNDED -->
- `InMemoryStateDb`: `{ accounts: HashMap<Address, DbAccount>, bytecodes: HashMap<B256, Bytecode>, block_hashes: HashMap<u64, B256> }` <!-- GROUNDED -->
- `impl DatabaseRef for InMemoryStateDb`: Read-only interface for EVM execution (Error = `StateError`). <!-- GROUNDED -->
- `impl Database for InMemoryStateDb`: Writable interface that delegates to `DatabaseRef`. <!-- GROUNDED -->
- `impl Default for InMemoryStateDb`: Provides an empty initial state. <!-- GROUNDED -->
- `impl Clone for InMemoryStateDb`: Derived for snapshotting. <!-- GROUNDED -->
- `StateError`: Implements `DBErrorMarker`, making it compatible with `revm` error handling. <!-- GROUNDED -->

## Functions & macros
- `InMemoryStateDb::new()`: Creates a new empty database. <!-- GROUNDED -->
- `InMemoryStateDb::with_genesis(HashMap<Address, GenesisAccount>)`: Initializes state with genesis accounts. <!-- GROUNDED -->
- `InMemoryStateDb::commit(&mut self, &BundleState)`: Applies state changes from execution results. <!-- GROUNDED -->
- `InMemoryStateDb::state_root() -> B256`: Computes deterministic root hash. <!-- GROUNDED -->
- `InMemoryStateDb::insert_block_hash(u64, B256)`: Records historical block hashes. <!-- GROUNDED -->
- `DatabaseRef::basic_ref(&self, Address) -> Option<AccountInfo>`: Returns account info if present. <!-- GROUNDED -->
- `DatabaseRef::storage_ref(&self, Address, U256) -> U256`: Returns storage value (zero if absent). <!-- GROUNDED -->

## Config schema
This crate has no configuration schema. It is a pure state storage component. <!-- GROUNDED -->

## Config defaults table
| Field | Type | Default | Source | Override path | Evidence |
|---|---|---|---|---|---|
| N/A | N/A | N/A | N/A | N/A | No configuration — pure state storage <!-- GROUNDED --> |

## Provider interfaces & swap points
- `DatabaseRef`: The primary interface for `revm` to read state during execution. <!-- GROUNDED -->
- `StateProvider`: Exposed via `TestStateDb` in the node binary, allowing access to `state_root()`. <!-- GROUNDED -->

## Feature flags & cfg
No custom feature flags are defined. <!-- GROUNDED -->

## SemVer & stability
Internal design. Breaking changes to `InMemoryStateDb` impact all execution and verification flows. <!-- PROPOSED -->

## Primary flows

### State Initialization
Genesis state is loaded via `with_genesis(HashMap<Address, GenesisAccount>)` at node startup, populating the initial account set from the chain specification. Each `GenesisAccount` is converted to a `DbAccount` with its `AccountInfo` and storage entries. <!-- GROUNDED -->

### State Root Derivation
The `state_root()` method iterates through sorted accounts and their sorted storage slots, hashing them with `keccak256` to produce a deterministic commitment. Empty states return `KECCAK_EMPTY`. This is a simplified hash, not an MPT (out of scope). <!-- GROUNDED -->

### State Commit
Execution results packaged as a `BundleState` are applied via `commit(&mut self, &BundleState)`. This handles account creation, updates, and destruction, as well as storage slot changes and bytecode insertions. <!-- GROUNDED -->

### Database Read
EVM execution reads from the state via the `DatabaseRef` implementation, which uses immutable references (`&self`). <!-- GROUNDED -->

## API omissions report

- **BLOCKER (INV-05)**: `finalize-to-commit` integration: There is no mechanism to trigger `commit()` after a block is finalized by the consensus engine. The `commit()` method exists but the call must be orchestrated externally. <!-- PROPOSED -->
- **UNKNOWN (INV-04)**: `Snapshot/Rollback`: While `Clone` is derived (enabling point-in-time copies), there is no explicit snapshot/rollback orchestration for managing state across verification boundaries. <!-- PROPOSED -->
- No persistent storage backend — all state is lost on process restart. <!-- GROUNDED -->
- `StateError` only has a single `Internal(String)` variant; richer error granularity may be needed. <!-- PROPOSED -->

## Open questions / TODOs

- **INV-03** (Verification Read-Only): **Grounded** — `DatabaseRef` uses `&self`, ensuring read-only access during EVM execution. <!-- GROUNDED -->
- **INV-04** (Snapshot Safety): `Clone` is derived (**grounded**), but runtime orchestration is **UNKNOWN**. <!-- PROPOSED -->
- **INV-05** (Commit Atomicity): `commit()` method exists (**grounded**), but finalize-to-commit integration is a **BLOCKER**. <!-- PROPOSED -->
- **INV-06** (Root Consistency): `state_root()` is deterministic via sorted keccak256 — **grounded**. <!-- GROUNDED -->
- **UNKNOWN**: How will large state growth be handled in-memory? No eviction or disk-backed fallback exists. <!-- PROPOSED -->
- **UNKNOWN**: Thread-safety of `commit()` — currently `&mut self`, protected by `Arc<RwLock<>>` at the node level. <!-- GROUNDED -->
