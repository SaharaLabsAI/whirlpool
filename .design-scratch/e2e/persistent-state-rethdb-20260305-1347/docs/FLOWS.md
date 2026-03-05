# FLOWS

## Scope and assumptions

This document defines the architecture-level execution flows for the persistent state feature (`state-reth` + `whirlpool-node` wiring), with explicit caller/callee edges, data in/out contracts, and error propagation.

- Domain references use `DOMAINS.md` labels:
  - `D1` = Persistent Storage
  - `D2` = State Interface
  - `D3` = State Root
  - `D4` = Node Wiring
- Hard blockers that constrain exact implementation details are called out inline (`BLK-001`, `BLK-002`, `BLK-003`).

---

## 1) Database Initialization Flow (node startup wiring)

**Domains:** `D4 -> D1 -> D2`

**Caller:** `whirlpool-node` startup path (`main.rs` runtime bootstrap)

**Primary callees:**
- `state_reth::init::create_db(path)`
- `state_reth::init::init_db(env)`
- `state_reth::db::RethStateDb` constructor/open path
- `EvmApplication<DB>` constructor path
- `EthRpcContext<S>` constructor path

**Data in:**
- Node config (MDBX DB path)
- Runtime initialization context (services to wire)

**Data out:**
- `Arc<RwLock<RethStateDb>>` shared backend
- EVM application instance bound to persistent state provider
- RPC context bound to persistent state provider

**Steps:**
1. `whirlpool-node::main` parses configuration and resolves the database path from node config (`D4`).
2. `whirlpool-node::main` calls `state_reth::init::create_db(path)` to open/create MDBX environment (`D1`).
3. `state_reth::init::create_db` returns `DatabaseEnv`; node calls `state_reth::init::init_db(&env)` to initialize required table schema/trie tables (`D1`, `D3` prereq).
4. `whirlpool-node::main` constructs `RethStateDb` from initialized environment (direct constructor or `open` helper) (`D2` backed by `D1`).
5. Node wraps backend as `Arc<RwLock<RethStateDb>>` for shared concurrent access (`D4`).
6. Node injects shared state into `EvmApplication` construction path (`D4 -> D2` consumer edge).
7. Node injects shared state into `EthRpcContext` construction path (`D4 -> D2` consumer edge).
8. Runtime services start with persistent backend as active state implementation.

**Error paths:**
- Config parse/path invalid -> startup abort with node-level init error (`D4`).
- `create_db` MDBX open/create failure -> mapped to `RethStateError::Init`/`Database` -> startup abort (`D1 -> D2 -> D4`).
- `init_db` schema/trie init failure -> mapped to `RethStateError::Init` -> startup abort (`D1 -> D4`).
- Backend construction failure -> propagated as `StateDb::Error` and treated as fatal startup error (`D2 -> D4`).
- Host prerequisite failure (BLK-003) -> startup abort per fatal initialization policy (`D4`).

---

## 2) Genesis Bootstrap Flow (first startup)

**Domains:** `D4 -> D2 -> D1 -> D3`

**Caller:** `whirlpool-node` startup path on first-run / empty-state detection

**Primary callees:**
- Empty DB detection logic in node + `state-reth` table probe path
- `state::StateDb::with_genesis(genesis_accounts)` implemented by `state_reth::db::RethStateDb`
- `state_reth::tables` write helpers for `PlainAccountState`, `PlainStorageState`, `Bytecodes`
- `state_reth::trie` state-root computation path

**Data in:**
- `HashMap<Address, GenesisAccount>`
- Initialized MDBX environment

**Data out:**
- Persisted genesis account/storage/code rows in MDBX
- Initial trie/state root (`B256`) persisted/available for subsequent reads
- Durable committed write transaction

**Steps:**
1. `whirlpool-node::main` checks whether DB is empty/uninitialized (first startup probe over state tables) (`D4`, `D1`).
2. If empty, node calls `RethStateDb::with_genesis(genesis_accounts)` through `StateDb` contract (`D4 -> D2`).
3. `RethStateDb::with_genesis` opens a write transaction (`DbTxMut`) (`D1`).
4. For each genesis account entry, `tables` layer writes account core fields to `PlainAccountState` (`D1`).
5. For each non-empty storage slot, `tables` layer writes entries to dupsort `PlainStorageState` (`D1`).
6. For accounts with code, `tables` layer writes bytecode keyed by code hash into `Bytecodes` (`D1`).
7. `RethStateDb::with_genesis` invokes trie/state-root computation path (`state_reth::trie`, via overlay root contract) to derive initial root (`D3`, constrained by BLK-002 details).
8. Transaction commits; initialization returns ready backend/state root to caller (`D1 -> D2 -> D4`).

**Error paths:**
- Empty-DB detection read failure -> startup abort (treat as initialization failure) (`D1 -> D4`).
- Any table write failure (account/storage/code) -> transaction rollback on drop -> error propagate as `RethStateError::Database` (`D1 -> D2`).
- Trie root computation failure -> `RethStateError::StateRoot` -> rollback/abort genesis bootstrap (`D3 -> D2 -> D4`).
- Final commit failure -> durability not guaranteed; propagate error and abort startup (`D1 -> D4`).

---

## 3) Transaction Execution Flow (read path)

**Domains:** `D2 -> D1` (with EVM/RPC consumers from `D4` wiring)

**Caller:** EVM execution engine (`revm::DatabaseRef`/`Database`) and RPC read handlers via `StateDb`

**Primary callees:**
- `RethStateDb::get_account(address)`
- `RethStateDb::get_storage(address, index)`
- `RethStateDb::get_code_by_hash(hash)`
- `state_reth::tables::*` read helpers and `state_reth::codec::*`

**Data in:**
- Address (`Address`)
- Storage key (`U256`)
- Code hash (`B256`)

**Data out:**
- Account: `Result<Option<AccountInfo>, StateDb::Error>`
- Storage value: `Result<U256, StateDb::Error>`
- Bytecode: `Result<Bytecode, StateDb::Error>`

**Steps (account read):**
1. EVM/RPC consumer calls `StateDb::get_account(addr)` (`D2` contract entry).
2. `RethStateDb::get_account` opens MDBX read transaction (`tx_read`) (`D1`).
3. `tables` path performs `get_by_encoded_key`/typed `get` against `PlainAccountState(addr)` (`D1`).
4. Raw reth account model is decoded/translated in `codec` layer to revm `AccountInfo` (`D2`).
5. Method returns `Option<AccountInfo>` to caller; read transaction drops/auto-aborts (`D1 -> D2`).

**Same-pattern steps (storage read):**
1. Caller invokes `StateDb::get_storage(address, index)`.
2. `RethStateDb` opens read tx.
3. `tables` dupsort read on `PlainStorageState` with `(address, index)` subkey seek.
4. Decode/map to `U256`, default zero when key absent (contract-specific behavior).
5. Return value; tx closes.

**Same-pattern steps (code read):**
1. Caller invokes `StateDb::get_code_by_hash(hash)`.
2. `RethStateDb` opens read tx.
3. `tables` typed read from `Bytecodes(hash)`.
4. `codec` translates stored bytecode model to revm `Bytecode`.
5. Return bytecode; tx closes.

**Error paths:**
- MDBX read transaction open failure -> `RethStateError::Database` -> caller receives `Err` (`D1 -> D2`).
- Table decode/codec failure -> `RethStateError::Codec` (`D1/D2`).
- Missing entries map to contract-defined non-error values where applicable (`None`/zero/default bytecode), not transport errors.

---

## 4) State Commit Flow (write path)

**Domains:** `D2 -> D1 -> D3`

**Caller:** Block execution completion path (`revm::Database::commit` -> `StateDb::commit`)

**Primary callees:**
- `RethStateDb::commit(bundle: &BundleState)`
- `state_reth::tables` write helpers
- `state_reth::trie` update/recompute helpers

**Data in:**
- `BundleState` containing account/storage/code deltas from executed block

**Data out:**
- Durable table updates in MDBX
- Updated trie-related state and resulting state root (`B256`), if computed in commit path
- `Result<(), StateDb::Error>` (or extended return if contract later evolves)

**Steps:**
1. EVM execution finalization calls `StateDb::commit(&bundle)` on shared backend (`D2`).
2. `RethStateDb::commit` acquires write path and opens MDBX write transaction (`tx_write`) (`D1`).
3. Iterate bundle account changes; write `PlainAccountState` rows via `tables` helpers (`D1`).
4. Iterate bundle storage changes; upsert dupsort `PlainStorageState` entries via cursor helpers (`D1`).
5. Iterate new/changed code blobs; upsert `Bytecodes` keyed by code hash (`D1`).
6. Invoke trie update/root path to reconcile trie-backed tables and compute new root (overlay semantics) (`D3`, exact normalization constrained by BLK-002).
7. Commit MDBX write transaction for durability; return success (`D1 -> D2`).

**Error paths:**
- Write transaction open failure -> `RethStateError::Database`.
- Any per-row account/storage/code write failure -> immediate error, tx not committed (rollback on drop).
- Trie update/root failure -> `RethStateError::StateRoot`, tx rollback, no partial durability.
- Commit failure -> error propagate; caller treats block application as failed (recovery policy at node/execution layer).

---

## 5) State Root Computation Flow

**Domains:** `D2 -> D3 -> D1`

**Caller:** Any consumer requesting `StateDb::state_root()` (node consistency checks, RPC exposure, post-commit verification)

**Primary callees:**
- `RethStateDb::state_root()`
- `state_reth::trie::compute_hashed_state(...)` (or equivalent helper)
- `reth_trie::StateRoot::overlay_root(tx, hashed_state)`

**Data in:**
- Current persisted state in `PlainAccountState` + `PlainStorageState`
- Trie tables (`AccountsTrie`, `StoragesTrie`)

**Data out:**
- Canonical Ethereum-style state root `B256`

**Steps:**
1. Caller invokes `StateDb::state_root()` (`D2` boundary).
2. `RethStateDb::state_root` opens MDBX read transaction (`D1`).
3. Trie helper scans account/storage state and hashes account + storage keys into `HashedPostState` overlay input (`D3`, contract specifics gated by BLK-002).
4. `StateRoot::overlay_root(tx, hashed_state)` executes trie walk over overlay + trie tables (`D3` using `D1` reads).
5. Computed Merkle root (`B256`) is returned through `StateDb` contract; tx closes (`D3 -> D2`).

**Error paths:**
- Read transaction open failure -> `RethStateError::Database`.
- Hashed-state construction failure (encoding/normalization) -> `RethStateError::StateRoot` or `Codec`.
- `overlay_root` failure -> `RethStateError::StateRoot`.
- Error bubbles as `StateDb::Error` to caller; no state mutation occurs.

---

## 6) Error Propagation Flow (MDBX -> consumer)

**Domains:** `D1 -> D2 -> D4` (+ app/rpc consumer surfaces)

**Caller:** Any read/write/state-root/genesis operation that touches MDBX

**Primary callees:**
- MDBX/reth-db operation (`tx_read`, `tx_write`, `get`, `put`, `commit`, cursor ops)
- `From<DatabaseError> for RethStateError` mapping in `state_reth::error`
- `StateDb` method returning `Result<..., StateDb::Error>`
- Consumer-specific handling (`app-evm` execution error path, `rpc-eth` error mapping path)

**Data in:**
- Low-level `DatabaseError` / codec/trie failures

**Data out:**
- `RethStateError::{Database|Codec|StateRoot|Init}`
- `Err(StateDb::Error)` observed by EVM/RPC/node caller
- Consumer-level mapped error response (execution abort, RPC JSON-RPC error, startup failure)

**Steps:**
1. Storage operation fails in MDBX/reth-db layer (`D1`): e.g., I/O error on tx open/read/write/commit.
2. `state-reth` maps source error into `RethStateError` variant (`D2` implementation surface).
3. Active `StateDb` method returns `Err(Self::Error)` to immediate caller (`D2` contract).
4. If caller is EVM execution path, error propagates through `revm::Database` error channel and aborts current execution/commit (`D4` consumer edge).
5. If caller is RPC path, error maps to JSON-RPC failure response with domain-appropriate code/message (`D4` consumer edge).
6. If caller is node startup/init path, error is treated as fatal initialization failure and aborts process startup (`D4`).

**Error path constraints:**
- BLK-001: final `StateDb` fallible signature/bounds must remain canonical across crates.
- BLK-102: exact error variant granularity may refine during implementation, but top-level categories and propagation semantics stay stable.

---

## Coverage check

The six required flows are covered with:
- explicit caller/callee chains,
- step-by-step module/function handling,
- data in/data out contracts,
- and concrete error propagation paths tied to domain boundaries.
