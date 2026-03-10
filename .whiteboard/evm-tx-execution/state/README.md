# Crate Contract — state

## 1. Purpose

In-memory EVM state database implementing revm's `Database` and `DatabaseRef` traits. Provides account/storage/bytecode reads for EVM execution and commit/state-root functionality for post-execution state management.

**Secondary change target**: verify `commit()` correctness for real `BundleState` inputs; potentially add `Clone` for snapshot support.

## 2. Public API

| Symbol | Kind | Signature | Status |
|---|---|---|---|
| `InMemoryStateDb` | struct | HashMap-based: accounts, bytecodes, block_hashes | Grounded |
| `InMemoryStateDb::new` | method | `() -> Self` | Grounded |
| `InMemoryStateDb::with_genesis` | method | `(alloc: &BTreeMap<Address, GenesisAccount>) -> Self` | Grounded |
| `InMemoryStateDb::commit` | method | `(&mut self, bundle: &BundleState)` | Grounded |
| `InMemoryStateDb::state_root` | method | `(&self) -> B256` | Grounded |
| `InMemoryStateDb::insert_block_hash` | method | `(&mut self, number: u64, hash: B256)` | Grounded |
| `StateError` | struct | Unit struct, impl Error + DBErrorMarker | Grounded |
| `DbAccount` | struct | `{ info: AccountInfo, storage: HashMap<U256, U256> }` | Grounded |

**Trait impls**: `Database<Error=StateError>`, `DatabaseRef<Error=StateError>`

## 3. Dependencies

**External**: alloy-primitives, alloy-genesis, revm (Database, DatabaseRef, BundleState, AccountInfo, Bytecode)

**No internal crate dependencies.**

## 4. Changes Required

### [PROPOSED] Add `Clone` derive or manual impl

Required for snapshot-based propose flow. `InMemoryStateDb` holds `HashMap`s which are `Clone`. Either `#[derive(Clone)]` or manual impl.

### Verify commit() correctness

**Current** (Grounded: `crates/state/src/db.rs::InMemoryStateDb::commit`):
- Iterates `bundle.state`: destroy → clear account+storage; create/update → set nonce/balance/code_hash + apply storage changes
- Iterates `bundle.contracts`: insert bytecodes
- Does NOT process reverts or logs (acceptable — reverts not needed for forward-only execution)

**Known gaps**:
- No bytecode cleanup on account destroy (minor — leaked bytecodes in memory)
- `state_root()` is flat keccak256, NOT Merkle Patricia Trie (documented out-of-scope)

### No changes expected to Database/DatabaseRef impls

Existing impls are correct for EVM execution reads.

## 5. Test Seams

| Test | Type | Boundary |
|---|---|---|
| commit correctly applies account create/update/destroy | Unit | Real BundleState from EVM |
| commit correctly applies storage changes | Unit | Real BundleState with storage diffs |
| commit correctly inserts bytecodes | Unit | BundleState with new contracts |
| state_root changes after commit | Unit | Before/after comparison |
| clone produces independent copy | Unit | Mutate clone, verify original unchanged |
| Database trait returns committed state | Unit | Commit then read via Database |
