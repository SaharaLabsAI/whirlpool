# Design Summary: Persistent State via reth-db (MDBX)

## Executive Summary

This design adds durable state persistence to Whirlpool by introducing a new `state-reth` crate that implements the existing `StateDb` trait using `reth-db` (MDBX). The integration replaces the in-memory `TestStateDb` wrapper in `whirlpool-node` with a persistent backend, while preserving generic trait boundaries for EVM execution (`app-evm`) and RPC serving (`rpc-eth`).

**Key decision:** Make the `StateDb` trait fallible to safely propagate MDBX I/O errors, requiring coordinated migration across `state`, `state-memory`, `app-evm`, `rpc-eth`, and `whirlpool-node`.

**Scope:** Module-level integration affecting 3 crates (1 new, 2 modified). No full-system redesign.

---

## Design Intent

**Objective:** Add persistent node state storage backed by `reth-db` (MDBX) by introducing a new `state-reth` crate that implements the existing `StateDb` trait, then wire `whirlpool-node` to use this persistent backend instead of the in-memory `TestStateDb` wrapper.

**Success criteria:**
1. `state-reth` crate builds and implements all `StateDb` methods over MDBX
2. `StateDb` trait is fallible; `state-memory` adapted to new signature
3. `state_root` returns trie-based root via `reth-trie` (not in-memory keccak256)
4. `commit` writes durably to MDBX with transaction rollback on error
5. `revm::Database` + `revm::DatabaseRef` implemented for EVM execution
6. `whirlpool-node` starts with persistent state; genesis initialization succeeds
7. Full integration test: EVM execution + RPC queries over persistent backend
8. Concurrency test: multiple readers + single writer under `Arc<RwLock<...>>`

---

## Key Architectural Decisions

### 1. Trait Fallibility Migration (BLK-001)

**Decision:** Make `StateDb` trait fallible with associated `Error` type.

**Rationale:**
- MDBX operations are fallible (I/O errors, transaction conflicts)
- Infallible trait + fallible I/O forces panic-on-error or silent failure
- `revm::Database` is already fallible; EVM execution expects error handling
- Consumers (`app-evm`, `rpc-eth`) already handle `revm::Database` errors

**Impact:**
- `state` crate: Add `type Error: std::error::Error + Send + Sync + 'static`; all methods return `Result<T, Self::Error>`
- `state-memory`: Add `type Error = core::convert::Infallible`; wrap returns in `Ok(...)`
- `state-reth`: Add `type Error = RethStateError`; propagate MDBX/codec/trie errors
- Consumers: Propagate `StateDb::Error` at call sites; map to execution/RPC error responses

### 2. State Root Semantics (BLK-002)

**Decision:** Adopt reth trie semantics (`StateRoot::overlay_root`) instead of current in-memory keccak256 approach.

**Rationale:**
- Trie root is Ethereum-canonical; aligns with reth/geth semantics
- Enables cross-client state verification
- Leverages vendored reth trie infrastructure (`reth-trie`, trie tables)

**Impact:**
- State roots will differ from `state-memory` for identical logical state
- Test strategy must validate trie root correctness independently (not against in-memory baseline)
- `state_root()` implementation in `state-reth` uses `reth_trie::StateRoot::overlay_root` with hashed state input

**Contract:**
- Root source: `PlainAccountState` + `PlainStorageState` tables
- Normalization: Exclude zero-value storage slots; follow reth compact codec boundaries
- Algorithm: `overlay_root` over trie tables (`AccountsTrie`, `StoragesTrie`)

### 3. Concurrency Model (MDBX Transaction Strategy)

**Decision:** Hold MDBX environment in `RethStateDb`, acquire transactions per method call.

**Rationale:**
- MDBX tx handles are not `Send`/`Sync`; environment handle is `Send + Sync`
- MDBX allows concurrent read txs but enforces single writer
- Per-method tx acquisition minimizes lock-hold time and prevents cross-thread leakage

**Implementation:**
- `RethStateDb` contains `Arc<DatabaseEnv>` (shared, thread-safe)
- Read methods: acquire short-lived read tx, auto-abort on drop
- Write methods: acquire short-lived write tx, commit on success, rollback on error
- Node wiring: `Arc<RwLock<RethStateDb>>` serializes access at Rust level (RwLock write guard + MDBX write tx enforce single writer)

### 4. Table Mapping Strategy

**Decision:** Use raw `reth-db` table access (not `reth-provider`).

**Rationale:**
- Direct table mapping (`PlainAccountState`, `PlainStorageState`, `Bytecodes`) aligns cleanly with 10-method `StateDb` trait
- Lower dependency overhead than full `reth-provider` stack
- Sufficient abstraction for current module-level integration

**Mapping:**
- `get_account` / `insert_account` → `PlainAccountState`
- `get_storage` / storage writes in `commit` → `PlainStorageState` (dupsort cursor)
- `get_code_by_hash` / code writes in `commit` → `Bytecodes`
- `get_block_hash` / `insert_block_hash` → `CanonicalHeaders` (resolved decision)
- `state_root` → reads account/storage tables + trie tables via `reth-trie` APIs

### 5. Error Handling and Propagation

**Decision:** Four-tier error taxonomy in `RethStateError`:
1. `Database(DatabaseError)` — MDBX I/O errors
2. `Init(String)` — DB creation/initialization failures
3. `Codec(String)` — revm <-> reth type conversion failures
4. `StateRoot(String)` — Trie computation failures

**Propagation path:**
- Storage layer (`D1`) → State Interface (`D2`) → Node Wiring (`D4`) → Consumers (EVM/RPC)
- Startup failures: fatal, abort node start
- Runtime failures: propagate via `StateDb::Error` and `revm::Database::Error`
- Write failures: transaction rollback on drop, no partial durability

---

## Crate Structure

### New Crate: `state-reth`

**Purpose:** Persistent `StateDb` implementation backed by MDBX.

**Modules:**
- `db.rs`: `RethStateDb` struct, `StateDb` impl, `revm::Database`/`DatabaseRef` impls
- `tables.rs`: Typed table access helpers, key encoding, cursor management
- `codec.rs`: revm <-> reth type conversion (`AccountInfo`, `Bytecode`, `StorageEntry`)
- `trie.rs`: State root computation via `StateRoot::overlay_root`
- `init.rs`: DB creation/initialization, genesis bootstrap helpers
- `error.rs`: `RethStateError` enum, error conversions

**Dependencies:**
- Workspace: `state`
- External/vendored: `reth-db`, `reth-db-api`, `reth-storage-errors`, `reth-codecs`, `reth-trie`, `revm`, `alloy-primitives`, `thiserror`

### Modified Crate: `state`

**Changes:**
- Add associated `type Error` to `StateDb` trait
- Change all trait methods to return `Result<T, Self::Error>`
- Preserve method names and conceptual behavior

**Impact:** Breaking source compatibility; requires consumer migration.

### Modified Crate: `whirlpool-node`

**Changes:**
- Add `NodeStateDbConfig` for MDBX path/args configuration
- Replace `Arc<RwLock<TestStateDb>>` with `Arc<RwLock<RethStateDb>>`
- Add `build_state_db` helper for startup sequence: `create_db` → `init_db` → `with_genesis`
- Add `NodeStartupError` for fatal initialization failures

**Impact:** Startup behavior changes; runtime backend is persistent.

---

## Critical Flows

### Flow 1: Database Initialization (Node Startup)
**Path:** `whirlpool-node::main` → `state_reth::create_db` → `state_reth::init_db` → `RethStateDb` constructor → wire into EVM app + RPC context

**Failure policy:** Startup abort on any error.

### Flow 2: Genesis Bootstrap (First Startup)
**Path:** `whirlpool-node` detects empty DB → `RethStateDb::with_genesis` → write genesis accounts/storage/code to MDBX → compute initial trie root → commit

**Idempotency:** First-run detection prevents duplicate initialization.

### Flow 3: Transaction Execution (Read Path)
**Path:** EVM/RPC consumer → `StateDb::get_account` → `RethStateDb` opens read tx → table read → codec translation → return `AccountInfo`

**Error handling:** Propagate `RethStateError::Database` or `Codec` to caller.

### Flow 4: State Commit (Write Path)
**Path:** Block execution → `StateDb::commit(BundleState)` → `RethStateDb` opens write tx → write account/storage/code tables → update trie → commit tx

**Durability:** Atomic commit or rollback; no partial writes visible.

### Flow 5: State Root Computation
**Path:** Caller → `StateDb::state_root` → `RethStateDb` reads account/storage tables → compute hashed state → `StateRoot::overlay_root` over trie tables → return `B256`

**Determinism:** Same logical state produces same trie root.

### Flow 6: Error Propagation
**Path:** MDBX error → `RethStateError` → `StateDb::Error` → EVM execution abort or RPC error response or node startup failure

**Recovery:** Caller decides policy (abort, retry, error response).

---

## Risk Summary and Mitigation

### Risk 1: State Root Parity (BLK-002)
**Risk:** Trie root semantics differ from in-memory baseline.

**Mitigation:**
- Validate trie root against Ethereum test vectors (not in-memory baseline)
- Document semantic change in migration notes
- Use separate test fixtures for trie-backed correctness

### Risk 2: MDBX Concurrency (Thread Safety)
**Risk:** MDBX tx handles are not `Send`/`Sync`; cross-thread leakage causes undefined behavior.

**Mitigation:**
- Enforce per-method tx acquisition: acquire, use, drop within single method call
- Never store tx handles in struct fields
- Add concurrency stress tests: 10 readers + 1 writer under `Arc<RwLock<...>>`

### Risk 3: Trait Fallibility Migration (BLK-001)
**Risk:** Breaking change for all `StateDb` consumers.

**Mitigation:**
- Update consumers (`app-evm`, `rpc-eth`) before testing
- Provide clear migration guide in `state` crate README
- Use `core::convert::Infallible` in `state-memory` to express "never fails"

### Risk 4: Codec Translation Bugs
**Risk:** revm <-> reth type conversion introduces data corruption.

**Mitigation:**
- Property tests: round-trip codec translation (`AccountInfo` → reth `Account` → `AccountInfo`)
- Validate against state-memory baseline for simple cases before divergence
- Unit tests for each codec function

### Risk 5: MDBX Host Prerequisites (BLK-003)
**Risk:** Build fails without C compiler + clang/bindgen; runtime fails without writable DB path.

**Mitigation:**
- Document prerequisites explicitly in `state-reth` README and `whirlpool-node` README
- Add startup validation for DB path writability
- Fatal error on prerequisite failure with clear error message

---

## Test Strategy

**Total test cases:** 46 (26 P0, 12 P1, 8 P2)

### Unit Tests (P0)
- Table access: insert/get account, storage, code, block hash
- State root: empty state root, determinism with accounts
- revm integration: `Database::basic`, `DatabaseRef::basic_ref`, storage reads
- Codec: round-trip translation for `AccountInfo`, `Bytecode`

### Integration Tests (P0)
- Durability: commit survives DB close/reopen
- Rollback: failed commit does not persist partial state
- Concurrency: 10 concurrent readers + 1 writer
- Genesis: `with_genesis` populates accounts/storage/code, computes root

### End-to-End Tests (P0)
- EVM execution → commit → restart → RPC query (balance reflects transfer)
- Genesis → EVM tx → RPC storage query (storage value matches committed value)
- State root consistency: commit → state_root → reopen → state_root (same root)

### Error Propagation Tests (P1)
- MDBX read failure → execution abort
- MDBX write failure → commit rollback

---

## Known Blockers

### Hard Blockers (Must Resolve Before Implementation)

| ID | Description | Resolution |
|----|-------------|------------|
| BLK-001 | `StateDb` fallibility contract | Resolved: associated `Error` type + `Result` returns for all methods |
| BLK-002 | Trie root input/normalization contract | Resolved: `PlainAccountState` + `PlainStorageState` tables, zero-value exclusion, `overlay_root` algorithm |
| BLK-003 | MDBX host prerequisites contract | Resolved: C compiler + clang/bindgen at build; writable DB path at runtime; fatal startup failure on missing |

### Soft Blockers (Refine During Implementation)

| ID | Description | Resolution Strategy |
|----|-------------|---------------------|
| BLK-101 | Block-hash table selection | Choose during implementation; `CanonicalHeaders` recommended |
| BLK-102 | Final error variant taxonomy | Refine during implementation; preserve top-level categories |
| BLK-103 | Performance strategy (caching/batching) | Start correctness-first; profile-guided optimization later |

---

## Implementation Phases

### Phase 1: Trait Migration (Foundation)
1. Update `StateDb` trait in `state` crate (associated error, fallible methods)
2. Migrate `state-memory` (add `type Error = Infallible`, wrap returns)
3. Update consumers (`app-evm`, `rpc-eth`) to propagate errors

### Phase 2: `state-reth` Core (Persistence)
1. Create crate skeleton + `RethStateError`
2. Implement `create_db`, `init_db` helpers
3. Implement read methods: `get_account`, `get_storage`, `get_code_by_hash`, `get_block_hash`
4. Implement codec layer (`AccountInfo` <-> reth `Account`, etc.)
5. Unit tests for table access

### Phase 3: Write Path (Durability)
1. Implement `insert_account`, `insert_block_hash`
2. Implement `commit` with `BundleState` application
3. Integration tests for durability and rollback

### Phase 4: State Root (Trie Integration)
1. Implement `state_root` via `StateRoot::overlay_root`
2. Implement `with_genesis` for initial state setup
3. Property tests for trie root determinism

### Phase 5: revm Integration (EVM Execution)
1. Implement `revm::DatabaseRef` (read-only)
2. Implement `revm::Database` (read-write)
3. Unit tests for EVM execution over persistent state

### Phase 6: Node Wiring (Production Integration)
1. Add `NodeStateDbConfig` in `whirlpool-node`
2. Update startup sequence: `build_state_db`, genesis initialization
3. Wire `Arc<RwLock<RethStateDb>>` into EVM app + RPC context
4. End-to-end tests: full node boot → tx → restart → verify

---

## Acceptance Criteria

Design is implementation-ready when:
1. All hard blockers (BLK-001, BLK-002, BLK-003) are resolved ✅
2. All crate contracts are documented in per-crate READMEs ✅
3. All flows have explicit caller/callee chains and error paths ✅
4. Test strategy covers all success criteria with P0 test cases ✅
5. Cross-crate dependencies are validated for version compatibility ✅
6. Concurrency model is explicit (tx lifetime, thread safety) ✅

**Status:** All acceptance criteria met. Design is ready for implementation.

---

## For Further Reading

- **Full design details:** See `INDEX.md` for navigation guide
- **Implementation contracts:** See `crates/*/README.md` for per-crate API surfaces
- **Execution flows:** See `FLOWS.md` for step-by-step caller/callee chains
- **Test specifications:** See `TESTS.md` for all 46 test cases with setup/assertions
- **Strategic decisions:** See `STRATEGY.md` for full rationale and alternatives considered

**Design iteration:** `persistent-state-rethdb-20260305-1347`  
**Total documentation:** 2,357 lines across 13 files
