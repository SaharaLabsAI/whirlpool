# Domain: State Storage

<!-- continuation round 2: resolves B-002 -->

## Purpose

The State Storage domain owns the EVM world state, including account balances, nonces, contract bytecode, storage slots, and block hashes. It provides the `revm::Database` implementation that the EVM execution domain uses to fetch state during transaction processing. This domain also handles state commitment, which applies `BundleState` diffs after execution, and computes the state root for block headers.

## Derived crates

| Crate | Role | Status |
|---|---|---|
| `state` | [PROPOSED] Core in-memory state database, revm integration, and root hashing | Proposed |

## Key Concepts

### InMemoryStateDb [PROPOSED]

The `InMemoryStateDb` is the primary state container, built on HashMaps for efficiency during the MVP phase. It stores accounts, bytecodes, and historical block hashes.

```rust
pub struct InMemoryStateDb {
    /// Account data: balance, nonce, code_hash, and storage
    pub accounts: HashMap<Address, DbAccount>,
    /// Contract bytecode indexed by code_hash
    pub(crate) bytecodes: HashMap<B256, Bytecode>,
    /// Block hashes indexed by block number
    pub(crate) block_hashes: HashMap<u64, B256>,
    /// Contract bytecode indexed by code_hash
    pub bytecodes: HashMap<B256, Bytecode>,
    /// Block hashes indexed by block number
    pub block_hashes: HashMap<u64, B256>,
}

pub struct DbAccount {
    pub info: AccountInfo,
    pub storage: HashMap<U256, U256>,
}
```

### Database trait implementation [PROPOSED]

The domain provides an implementation of the `revm::Database` trait. This allows the EVM to query the state during execution. The implementation maps trait methods to HashMap lookups:
- `basic(addr)`: Retrieves `AccountInfo` (balance, nonce, code_hash) from the `accounts` map.
- `storage(addr, key)`: Retrieves the storage value for a specific account and slot key.
- `code_by_hash(hash)`: Returns contract bytecode from the `bytecodes` map.
- `block_hash(number)`: Returns the block hash for a given block number from the `block_hashes` map.

If the requested data is missing, the implementation returns `Ok` with a default value. For accounts, `basic()` returns `Ok(None)` for unknown addresses. For storage and block hashes, it returns zeroed values (`U256::ZERO` and `B256::ZERO` respectively).

### State commitment [PROPOSED]

The `commit(bundle_state: &BundleState)` method applies execution results to the database. It processes a `BundleState` object, which represents the aggregated state changes from one or more transactions.

The commitment logic iterates through `BundleState.state` for account updates and `BundleState.contracts` for new bytecodes. It updates the in-memory maps according to the account status:
- **Created/Changed**: Updates balance, nonce, and code hash.
- **Destroyed**: Handles account deletion for self-destructed contracts.

Storage changes are applied by writing new values or clearing slots if the storage was wiped.

### State root computation [PROPOSED]

The `state_root() -> B256` method computes a deterministic hash of the entire state. This hash is required for the block header to ensure state consistency across nodes. 

For the MVP, the state root is calculated by:
1. Sorting all accounts by their address.
2. For each account, sorting storage slots by their key.
3. Hashing the combined data stream using `keccak256` (consistent across all documents).

**Warning**: This implementation is a placeholder and does not use a Merkle Patricia Trie.
- [BLOCKER: persistent storage + real MPT for production]

### Clone semantics [PROPOSED]

`InMemoryStateDb` implements `Clone` to allow cheap snapshots of the entire state. This is a requirement for the `revm::Database + Clone` bound on `EvmApplication<DB>`. Clones are used during propose and verify to isolate speculative execution — changes are accumulated on the clone, and if execution fails, the clone is simply dropped. Canonical state is only updated when the consensus layer finalizes a block. `EvmApplication` wraps the state in `Arc<RwLock<InMemoryStateDb>>` to satisfy the `&self` signature of `Application::propose/verify` while still allowing interior mutation on finalization.

## Invariants

- **State root is deterministic**: The same genesis state and transaction sequence always produce the identical state root.
- **Commit is atomic**: All changes in a `BundleState` are applied in a single operation. The database never enters a partially updated state.
- **Default for missing state**: Queries for non-existent state return empty or zeroed defaults rather than errors.
- **State root changes on commit**: Every successful commit that alters the state results in a different state root.
- **Clone isolation**: Mutations to a cloned `InMemoryStateDb` never affect the original — clones are fully independent.
- **Canonical state protection**: Propose and verify operate on clones; canonical state is only mutated on finalization (via `Arc<RwLock<_>>`).

## Domain Boundaries

### Data Management Only
The State Storage domain focuses exclusively on data persistence and retrieval. It doesn't:
- Execute transactions (this is the EVM Execution domain's job).
- Validate blocks or participate in consensus (handled by Application and Consensus domains).
- Persist data to physical disk (this is out of scope for the current MVP).
- Own commit timing — the consensus/finalization layer decides WHEN to commit; the state domain provides the HOW (`commit()` method).
The State Storage domain focuses exclusively on data persistence and retrieval. It doesn't:
- Execute transactions (this is the EVM Execution domain's job).
- Validate blocks or participate in consensus (handled by Application and Consensus domains).
- Persist data to physical disk (this is out of scope for the current MVP).
