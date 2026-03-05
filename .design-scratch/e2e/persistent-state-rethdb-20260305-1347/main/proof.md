# Proof: Persistent State with reth-db

## S0: Pre-conditions

This section establishes the foundational state that must be verified before implementation begins. Each pre-condition is grounded in existing code or explicit design decisions.

### PC-001: StateDb trait exists with 11 infallible methods

**Statement:** The `StateDb` trait is defined in `state/src/traits.rs` with exactly 11 methods, all currently infallible (no `Result` returns).

**Grounding:** `state/src/traits.rs` lines 8-25

**Methods:**
- `new() -> Self`
- `with_genesis(HashMap<Address, GenesisAccount>) -> Self`
- `state_root(&self) -> B256`
- `commit(&mut self, &BundleState)`
- `get_account(&self, Address) -> Option<AccountInfo>`
- `get_code_by_hash(&self, B256) -> Bytecode`
- `get_storage(&self, Address, U256) -> U256`
- `get_block_hash(&self, u64) -> B256`
- `insert_account(&mut self, Address, AccountInfo)`
- `insert_block_hash(&mut self, u64, B256)`

**Verification:** Read `state/src/traits.rs` and confirm trait signature matches.

**Impact:** Phase 1 implementation will modify this trait to add fallibility (associated `Error` type + `Result` returns).

---

### PC-002: InMemoryStateDb is behavioral reference implementation

**Statement:** The `state-memory` crate provides `InMemoryStateDb` struct that implements all `StateDb` methods using in-memory HashMaps. This serves as the behavioral baseline for state semantics.

**Grounding:** `state-memory/src/db.rs` lines 19-30 (struct definition), lines 31-38 (StateDb impl start)

**Storage model:**
- Accounts: `HashMap<Address, DbAccount>` (balance, nonce, code_hash, storage)
- Bytecodes: `HashMap<B256, Bytecode>`
- Block hashes: `HashMap<u64, B256>`

**Verification:** Run `cargo build -p state-memory` and confirm it compiles.

**Impact:** Phase 1 will require updating `InMemoryStateDb` to use `type Error = Infallible` and wrap all returns in `Ok(...)`.

---

### PC-003: Consumers are generic over S: StateDb

**Statement:** Both `app-evm` and `rpc-eth` are architected to consume `StateDb` implementations generically via trait bounds. The node wiring in `whirlpool-node` wraps concrete implementations in `Arc<RwLock<S>>` and passes them to consumers.

**Grounding:** 
- `whirlpool-node/src/main.rs` lines 27-76 (TestStateDb wrapper implementing StateDb)
- `whirlpool-node/src/main.rs` line 10 (`Arc<RwLock<...>>` pattern)
- INTENT.md line 12 (explicitly states consumers should remain implementation-agnostic)

**Verification:** Grep for `StateDb` trait bounds in `app-evm` and `rpc-eth` crates; confirm generic usage.

**Impact:** Trait fallibility migration (Phase 1) will require error propagation at consumer call sites.

---

### PC-004: Vendored reth-db crates are available with MDBX support

**Statement:** The `vendor/reth/` directory contains the full reth monorepo, including storage stack crates (`reth-db`, `reth-db-api`, `reth-storage-errors`, `reth-codecs`, `reth-trie`) required for MDBX-backed persistence.

**Grounding:** 
- `vendor/reth/` directory exists (verified via `ls`)
- STRATEGY.md lines 287-299 (proposed dependency paths reference `vendor/reth/crates/storage/...`)
- SHARED_CONTEXT.md documents reth-db API patterns

**Verification:** Confirm `vendor/reth/crates/storage/db/Cargo.toml` includes `mdbx` feature.

**Impact:** Phase 2 will add workspace dependencies from `state-reth` to these vendored crates.

---

### PC-005: Nix dev shell provides C compiler + clang for mdbx-sys

**Statement:** Building MDBX (via `mdbx-sys` FFI bindings) requires a C compiler and clang/bindgen toolchain. The workspace uses a Nix flake (`flake.nix`) to provide the development environment.

**Grounding:** 
- Root `flake.nix` exists (observed in directory listing)
- BLK-003 in BLOCKERS.md (MDBX host prerequisites blocker)
- SUMMARY.md lines 220-226 (Risk 5: MDBX Host Prerequisites)

**Verification:** Run `nix develop --command clang --version` and confirm clang is available. Check `flake.nix` for C toolchain dependencies.

**Assumption flag:** [ASSUMPTION] — `flake.nix` is assumed to provide C toolchain; must be verified during Phase 1.

**Impact:** If missing, `flake.nix` must be updated to include C compiler + clang dependencies before MDBX build succeeds.

---

### PC-006: whirlpool-node currently uses TestStateDb(InMemoryStateDb)

**Statement:** The current node binary wraps `InMemoryStateDb` in a `TestStateDb` passthrough struct and uses it as the state backend. This is the wiring path that Phase 6 will replace with `RethStateDb`.

**Grounding:** `whirlpool-node/src/main.rs` lines 27-76 (TestStateDb definition and StateDb impl)

**Verification:** Grep `whirlpool-node/src/main.rs` for `TestStateDb` instantiation and confirm it's the active backend.

**Impact:** Phase 6 will remove `TestStateDb` and wire `RethStateDb` directly.

---

### PC-007: StateDb consumers already handle revm::Database errors

**Statement:** EVM execution in `app-evm` already expects fallible `revm::Database` trait implementations (which return `Result<T, Self::Error>`). This means error propagation infrastructure exists in the execution path.

**Grounding:** 
- `whirlpool-node/src/main.rs` lines 79-100 (TestStateDb implements `revm::Database` with `type Error = state::StateError`)
- STRATEGY.md lines 86-87 (revm Database trait is fallible)
- STRATEGY.md lines 92-94 (consumers already handle revm errors)

**Verification:** Confirm `app-evm` execution path propagates `revm::Database::Error`.

**Impact:** Trait fallibility migration leverages existing error handling patterns; minimal new infrastructure needed.

---

### Pre-conditions Summary

| ID | Statement | Verification Method | Status |
|---|---|---|---|
| PC-001 | StateDb trait exists with 11 infallible methods | File read: `state/src/traits.rs` | ✅ Grounded |
| PC-002 | InMemoryStateDb is behavioral reference | Compile check: `cargo build -p state-memory` | ✅ Grounded |
| PC-003 | Consumers are generic over StateDb | Grep: trait bounds in app-evm, rpc-eth | ✅ Grounded |
| PC-004 | Vendored reth-db crates available | File check: `vendor/reth/crates/storage/db/` | ✅ Grounded |
| PC-005 | Nix shell provides C compiler + clang | Command: `nix develop --command clang --version` | ⚠️ Assumption |
| PC-006 | whirlpool-node uses TestStateDb wrapper | Grep: `whirlpool-node/src/main.rs` for TestStateDb | ✅ Grounded |
| PC-007 | Consumers handle revm::Database errors | Code read: app-evm execution error path | ✅ Grounded |

**Pre-conditions gate:** All grounded pre-conditions verified. PC-005 flagged as assumption requiring validation in Phase 1.

## S1: Split Justification

### Decision: This is a SINGLE intent (no sub-intent split)

This feature represents one atomic, coherent unit of work that should NOT be decomposed into separate sub-intents. The following analysis justifies this decision.

---

### Justification Criteria

#### 1. Single Coherent Goal

**Objective:** Add persistent state storage backed by reth-db (MDBX) to replace in-memory state.

**Why single intent:**
- The user's request is unambiguous: "make state survive restart using reth-db"
- All implementation work serves one outcome: state durability
- Success is binary: either state persists across restarts or it doesn't
- No meaningful intermediate milestones that deliver independent value

**Counter-example (rejected split):**
- Split A: "Make StateDb trait fallible"
- Split B: "Add state-reth crate"
- Split C: "Wire persistent backend"

**Why rejected:** Splits A and B deliver no user value independently. A fallible trait with no persistent implementation is meaningless. A persistent implementation without trait changes cannot be wired. Only when ALL pieces are complete does the feature work.

---

#### 2. Tightly Coupled Changes

**Affected crates:** `state` (trait), `state-reth` (impl), `whirlpool-node` (wiring)

**Coupling analysis:**

| Change | Depends On | Blocks |
|---|---|---|
| StateDb trait fallibility (state) | None | state-reth implementation, consumer migration |
| state-memory migration | StateDb trait fallibility | Consumer testing baseline |
| state-reth implementation | StateDb trait fallibility | Node wiring |
| Node wiring (whirlpool-node) | state-reth crate exists | End-to-end testing |
| Consumer migration (app-evm, rpc-eth) | StateDb trait fallibility | Compilation |

**Observation:** Changes form a **linear dependency chain**. No parallelizable sub-tasks exist that deliver independent value. Breaking this into separate intents creates incomplete intermediate states that cannot be validated.

---

#### 3. No Natural Decomposition Boundary

**Potential split points considered:**

**Option A: Split by layer**
- Sub-intent 1: Trait fallibility migration (state + state-memory + consumers)
- Sub-intent 2: Persistent implementation (state-reth)
- Sub-intent 3: Node wiring

**Rejected because:**
- Sub-intent 1 alone adds error handling for an in-memory backend — no persistence, no value
- Sub-intent 2 cannot be compiled without Sub-intent 1 complete
- Sub-intent 3 cannot be tested without Sub-intent 2 complete
- Total implementation time is identical (no parallelization gains)

**Option B: Split by feature surface**
- Sub-intent 1: Read-only persistence (get_account, get_storage, get_code_by_hash)
- Sub-intent 2: Write persistence (commit, insert_account, insert_block_hash)
- Sub-intent 3: State root + genesis

**Rejected because:**
- Read-only persistence cannot be tested meaningfully (no way to populate state without writes)
- Write persistence without state root breaks trie invariants
- Genesis initialization is required for first-run bootstrap (blocking all testing)
- This split creates artificial staging that increases coordination overhead

**Conclusion:** No natural decomposition exists that respects dependency ordering and delivers incremental value.

---

#### 4. Incomplete Intermediate States Are Non-Functional

**If we split this intent, intermediate states would be:**

**After "Trait Fallibility" sub-intent:**
- StateDb trait is fallible
- InMemoryStateDb wraps returns in `Ok(...)`
- Consumers propagate StateDb::Error
- **State:** Still in-memory, no persistence, feature incomplete
- **Value delivered:** Zero (only added boilerplate for future work)

**After "Persistent Implementation" sub-intent:**
- state-reth crate exists
- RethStateDb implements StateDb
- **State:** Crate compiles but not wired into node
- **Value delivered:** Zero (no way to test or use it)

**After "Node Wiring" sub-intent (final):**
- whirlpool-node uses RethStateDb
- State persists across restarts
- **State:** Feature complete
- **Value delivered:** 100% (feature now works)

**Observation:** Value delivery is step-function at the end. Intermediate milestones deliver 0% value.

---

#### 5. Testing Cannot Validate Partial States

**What tests validate:**

| Test Type | Validates | Requires |
|---|---|---|
| Unit tests (state-reth) | Table access, codec, state root | state-reth crate complete |
| Integration tests (durability) | State survives DB close/reopen | state-reth + commit impl complete |
| End-to-end tests | State survives node restart | Full wiring complete |
| Consumer tests (app-evm, rpc-eth) | Error propagation paths | Trait fallibility + consumers updated |

**Observation:** Most tests (integration, E2E) require the ENTIRE feature complete. Unit tests can run incrementally but don't validate the feature goal (durability).

---

#### 6. Coordination Overhead vs. Implementation Cost

**If split into 3 sub-intents:**
- Overhead: 3 separate design reviews, 3 proof sections, 3 acceptance gates, 3 context-switch penalties
- Benefit: Parallel work (but no parallelizable tasks exist due to linear dependency chain)

**If kept as single intent:**
- Overhead: 1 design review, 1 proof section, 1 acceptance gate
- Benefit: Clear ownership, no cross-intent coordination, faster iteration

**Estimated implementation time:**
- Trait migration: ~2 hours
- state-reth core: ~8 hours
- Node wiring: ~2 hours
- **Total:** ~12 hours (1.5 developer days)

**Conclusion:** For a 1.5-day feature with linear dependencies, splitting increases coordination cost without reducing implementation risk or time.

---

### Alternative Considered: Depth-First Incremental Implementation

**Strategy:** Implement one "vertical slice" (e.g., account reads) end-to-end before expanding to full StateDb surface.

**Rejected because:**
- MDBX initialization and table setup are all-or-nothing (can't incrementally bootstrap a database)
- State root computation requires full account+storage state (can't compute root with partial data)
- Genesis initialization must populate all account fields (balance, nonce, code, storage) atomically
- EVM execution requires full StateDb surface (cannot execute with partial backend)

**Conclusion:** This feature has no meaningful "vertical slice" that delivers a testable subset.

---

### Split Justification Summary

| Criterion | Single Intent | Multiple Sub-Intents |
|---|---|---|
| User goal coherence | ✅ One clear outcome | ❌ Artificial staging |
| Crate coupling | ✅ Linear dependency chain | ❌ Forces incomplete states |
| Natural decomposition | ❌ No natural boundaries | ❌ All splits are artificial |
| Intermediate value | ❌ Step-function delivery | ❌ 0% until final sub-intent |
| Test validation | ✅ Tests require full feature | ❌ Partial states untestable |
| Coordination overhead | ✅ Minimal (1 review) | ❌ High (3+ reviews) |
| Implementation time | ~12 hours (parallelization impossible) | ~12 hours (no time savings) |

**Verdict:** This is a SINGLE intent. Splitting would increase coordination overhead without reducing risk, enabling parallelization, or delivering incremental value.

---

### Scope Boundary Note

**What IS in scope (single intent):**
- Trait fallibility migration (state, state-memory, consumers)
- state-reth crate creation and full StateDb implementation
- Node wiring to use persistent backend
- Genesis initialization for first run
- Full test coverage (unit, integration, E2E)

**What is OUT of scope (future work, separate intents):**
- Performance optimization (caching, batching) — deferred per BLK-103
- State migration tools (in-memory to persistent) — not required (fresh genesis)
- Alternative backends (e.g., RocksDB) — out of scope
- State pruning/archival strategies — out of scope

**Rationale for out-of-scope:** These are orthogonal features that CAN be implemented independently after persistence is working. They don't block the core goal (state durability).

## S2: Invariants

This section defines the invariants that MUST hold throughout the implementation and operation of the persistent state feature. Each invariant is verified through a specific method and owned by a particular domain.

### INV-1: StateDb Trait Fallibility Contract

**Statement:** The `StateDb` trait in the `state` crate has an associated `Error` type and all methods return `Result<T, Self::Error>`.

**Verification Method:**
1. Compile-time: `cargo check -p state` succeeds with fallible trait signature
2. Static analysis: Trait definition in `state/src/traits.rs` includes `type Error: std::error::Error + Send + Sync + 'static`
3. Test: `TC-ST-U001` (test_statedb_trait_fallible_signature)

**Owning Domain:** State Interface (D2)

**Grounding:** STRATEGY.md lines 97-113 (trait modifications), BLOCKERS.md BLK-001 (resolution requirement)

---

### INV-2: InMemoryStateDb Error Infallibility

**Statement:** `InMemoryStateDb::Error` is `std::convert::Infallible` (or equivalent never-fails type), ensuring the reference implementation never produces runtime errors.

**Verification Method:**
1. Static check: `state-memory/src/db.rs` includes `type Error = Infallible`
2. Test: `TC-ST-U002` (test_state_memory_infallible_impl) - all operations return `Ok(_)`
3. Compile check: `cargo build -p state-memory` succeeds

**Owning Domain:** State Interface (D2)

**Grounding:** STRATEGY.md lines 116-117 (state-memory migration), PC-002 (baseline implementation status)

---

### INV-3: State Persistence Across Restarts

**Statement:** `RethStateDb` persists committed state to MDBX durably; data survives process restarts when the same database path is used.

**Verification Method:**
1. Integration test: `TC-SR-I001` (test_commit_durability) - close DB, reopen, verify reads
2. Manual verification: commit state → kill process → restart → query state
3. File system check: `data.mdb` file exists and contains data after commit

**Owning Domain:** Persistent Storage (D1)

**Grounding:** INTENT.md line 4 (add persistent storage), FLOWS.md section 4 (state commit flow), TESTS.md TC-SR-I001

---

### INV-4: State Root Determinism

**Statement:** `state_root()` is deterministic — the same logical state (same accounts, storage, code) always produces the same `B256` root hash, regardless of insertion order or process restarts.

**Verification Method:**
1. Property test: `TC-SR-I007` (test_state_root_determinism) - insert same state in two DBs with different order
2. Idempotency test: `TC-SR-I008` (test_state_root_idempotency) - multiple calls return same root
3. Restart test: `TC-CC-I004` - root before restart equals root after restart

**Owning Domain:** State Root (D3)

**Grounding:** STRATEGY.md lines 220-256 (state root strategy), TESTS.md TC-SR-I007, TC-SR-I008

---

### INV-5: Commit Atomicity

**Statement:** `commit(BundleState)` is atomic — either ALL changes (accounts, storage, code) are applied durably to MDBX, or NONE are (transaction rollback on error).

**Verification Method:**
1. Integration test: `TC-SR-I002` (test_commit_rollback_on_error) - inject error, verify no partial writes
2. Integration test: `TC-CC-I006` (MDBX write failure → commit rollback)
3. Manual verification: corrupt write path, attempt commit, verify clean rollback

**Owning Domain:** Persistent Storage (D1) + State Interface (D2)

**Grounding:** STRATEGY.md lines 194-203 (commit implementation), FLOWS.md section 4 (state commit flow error paths)

---

### INV-6: RethStateDb Thread Safety

**Statement:** `RethStateDb` implements `Clone + Send + Sync + Debug`, enabling safe sharing across threads via `Arc<RwLock<...>>` in the node runtime.

**Verification Method:**
1. Compile-time: Trait bounds compilation check in `whirlpool-node` wiring
2. Concurrency test: `TC-SR-I003` (test_concurrent_reads) - multiple threads access shared DB
3. Concurrency test: `TC-SR-I004` (test_single_writer_multiple_readers) - writer + readers under `Arc<RwLock<...>>`

**Owning Domain:** Persistent Storage (D1) + Node Wiring (D4)

**Grounding:** STRATEGY.md lines 163-218 (concurrency model), PC-003 (consumer thread-safety requirements)

---

### INV-7: Consumer Compilation Compatibility

**Statement:** All consumers (`app-evm`, `rpc-eth`) compile unchanged (or with minimal error propagation updates) against the new fallible `StateDb` trait. Generic trait bounds are satisfied by `RethStateDb`.

**Verification Method:**
1. Compile check: `cargo build -p app-evm` succeeds
2. Compile check: `cargo build -p rpc-eth` succeeds
3. Test: All consumer unit tests pass (`cargo test -p app-evm`, `cargo test -p rpc-eth`)

**Owning Domain:** State Interface (D2) + Node Wiring (D4)

**Grounding:** PC-003 (consumers are generic over StateDb), INTENT.md line 12 (maintain compatibility for consumers)

---

### INV-8: Genesis Bootstrap Correctness

**Statement:** Genesis accounts are correctly populated on first startup: all accounts, storage slots, and code from `with_genesis(genesis_alloc)` are durably written and readable via `StateDb` methods.

**Verification Method:**
1. Integration test: `TC-SR-I005` (test_with_genesis_populates_accounts) - verify all data readable
2. Integration test: `TC-WN-I002` (test_genesis_initialization_on_first_startup) - first run → genesis applied, restart → no duplicate init
3. End-to-end test: `TC-CC-I003` (Genesis → Commit → Read) - full flow validation

**Owning Domain:** Node Wiring (D4) + Persistent Storage (D1)

**Grounding:** FLOWS.md section 2 (Genesis Bootstrap Flow), TESTS.md TC-SR-I005, TC-WN-I002

---

### Invariants Summary Table

| ID | Statement | Verification | Domain | Priority |
|---|---|---|---|---|
| INV-1 | StateDb trait is fallible | Compile + test TC-ST-U001 | D2 | P0 |
| INV-2 | InMemoryStateDb uses Infallible error | Static check + test TC-ST-U002 | D2 | P0 |
| INV-3 | State persists across restarts | Test TC-SR-I001 + manual | D1 | P0 |
| INV-4 | state_root() is deterministic | Tests TC-SR-I007, TC-SR-I008, TC-CC-I004 | D3 | P0 |
| INV-5 | commit() is atomic | Tests TC-SR-I002, TC-CC-I006 | D1+D2 | P0 |
| INV-6 | RethStateDb is Clone+Send+Sync+Debug | Compile + tests TC-SR-I003, TC-SR-I004 | D1+D4 | P0 |
| INV-7 | Consumers compile with new trait | Compile checks + unit tests | D2+D4 | P0 |
| INV-8 | Genesis bootstrap is correct | Tests TC-SR-I005, TC-WN-I002, TC-CC-I003 | D4+D1 | P0 |

**All invariants are P0 (critical)** — violations block feature acceptance.

## S3: Acceptance Criteria

This section defines concrete, testable criteria that MUST be met for the persistent state feature to be considered complete. Each criterion is verifiable through specific commands or tests.

### Build and Compilation Criteria

#### AC-1: state-reth Crate Compilation

**Description:** The `state-reth` crate compiles successfully with all dependencies resolved.

**Verification:** `nix develop --command cargo build -p state-reth`

**Expected Result:** Exit code 0, no compilation errors.

**Priority:** P0 (Critical)

**Grounding:** STRATEGY.md section "Acceptance Criteria" line 379

---

#### AC-2: state-reth Unit Tests Pass

**Description:** All unit tests in `state-reth` pass, covering table access, codec translation, and error handling.

**Verification:** `nix develop --command cargo test -p state-reth`

**Expected Result:** All tests pass (0 failures).

**Priority:** P0 (Critical)

**Test IDs:** TC-SR-U001 through TC-SR-U017

**Grounding:** STRATEGY.md line 379, TESTS.md unit test section

---

#### AC-3: state Crate Compilation (Trait Migration)

**Description:** The `state` crate compiles with the fallible `StateDb` trait (associated `Error` type + `Result` returns).

**Verification:** `nix develop --command cargo test -p state`

**Expected Result:** All tests pass; trait signature is fallible.

**Priority:** P0 (Critical)

**Grounding:** STRATEGY.md lines 97-113 (trait modifications), BLOCKERS.md BLK-001

---

#### AC-4: state-memory Crate Adaptation

**Description:** `state-memory` crate compiles and tests pass with `InMemoryStateDb` updated to implement fallible `StateDb` trait using `Infallible` error type.

**Verification:** `nix develop --command cargo test -p state-memory`

**Expected Result:** All tests pass; `type Error = Infallible`.

**Priority:** P0 (Critical)

**Grounding:** STRATEGY.md lines 116-117, INV-2

---

#### AC-5: app-evm Consumer Compilation

**Description:** The `app-evm` crate compiles and tests pass with the new fallible `StateDb` trait (consumers updated for error propagation).

**Verification:** `nix develop --command cargo test -p app-evm`

**Expected Result:** All tests pass; no compilation errors.

**Priority:** P0 (Critical)

**Grounding:** INV-7, PC-003

---

#### AC-6: rpc-eth Consumer Compilation

**Description:** The `rpc-eth` crate compiles and tests pass with the new fallible `StateDb` trait.

**Verification:** `nix develop --command cargo test -p rpc-eth`

**Expected Result:** All tests pass; no compilation errors.

**Priority:** P0 (Critical)

**Grounding:** INV-7, PC-003

---

#### AC-7: whirlpool-node Wiring Compilation

**Description:** `whirlpool-node` compiles with `RethStateDb` wired as the persistent backend (replacing `TestStateDb`).

**Verification:** `nix develop --command cargo test -p whirlpool-node`

**Expected Result:** All tests pass; wiring is correct.

**Priority:** P0 (Critical)

**Grounding:** FLOWS.md section 1 (Database Initialization Flow)

---

#### AC-11: Full Workspace Build

**Description:** The entire workspace compiles successfully with all crates integrated.

**Verification:** `nix develop --command cargo build`

**Expected Result:** Exit code 0, all crates compile.

**Priority:** P0 (Critical)

**Grounding:** STRATEGY.md line 388

---

#### AC-12: Full Workspace Tests

**Description:** All workspace tests pass, including unit, integration, and end-to-end tests.

**Verification:** `nix develop --command cargo test`

**Expected Result:** All tests pass (0 failures).

**Priority:** P0 (Critical)

**Grounding:** STRATEGY.md line 388

---

### Functional Acceptance Criteria

#### AC-8: State Persistence Integration Test

**Description:** Write state to MDBX → close database → reopen database → read state → verify data matches.

**Verification:** Integration test `TC-SR-I001` (test_commit_durability)

**Test Steps:**
1. Create and initialize MDBX database
2. Insert accounts via `commit(BundleState)`
3. Close database (drop `RethStateDb`)
4. Reopen database at same path
5. Read accounts via `get_account`
6. Assert: all committed accounts are readable

**Expected Result:** Test passes; state survives database close/reopen.

**Priority:** P0 (Critical)

**Grounding:** INV-3, FLOWS.md section 4

---

#### AC-9: Genesis Bootstrap Integration Test

**Description:** Genesis initialization populates state on first startup; state root matches expected value.

**Verification:** Integration tests `TC-SR-I005`, `TC-SR-I006`, `TC-WN-I002`

**Test Steps:**
1. Create empty MDBX database
2. Call `with_genesis(genesis_alloc)` with predefined accounts
3. Verify all accounts readable via `get_account`
4. Verify storage readable via `get_storage`
5. Verify code readable via `get_code_by_hash`
6. Compute `state_root()` and verify it differs from empty state root

**Expected Result:** Tests pass; genesis data is persisted and readable.

**Priority:** P0 (Critical)

**Grounding:** INV-8, FLOWS.md section 2

---

#### AC-10: BundleState Commit Integration Test

**Description:** Commit `BundleState` with account/storage/code changes → verify changes are visible in subsequent reads.

**Verification:** Unit tests `TC-SR-U003`, `TC-SR-U005`, `TC-SR-U007` + integration test `TC-SR-I001`

**Test Steps:**
1. Create `BundleState` with account deltas, storage writes, and code
2. Call `commit(bundle)`
3. Read back via `get_account`, `get_storage`, `get_code_by_hash`
4. Assert: all changes are visible

**Expected Result:** Tests pass; committed state is readable.

**Priority:** P0 (Critical)

**Grounding:** INV-5, FLOWS.md section 4

---

### Quality Assurance Scenarios

#### QA-1: Concurrent Reads During Write

**Description:** Multiple concurrent read operations during a write operation do not panic or deadlock.

**Verification:** Integration test `TC-SR-I004` (test_single_writer_multiple_readers)

**Test Steps:**
1. Wrap `RethStateDb` in `Arc<RwLock<...>>`
2. Spawn writer task (acquires write lock, commits `BundleState`)
3. Spawn 5 reader tasks (acquire read locks, call `get_account`)
4. Join all tasks
5. Assert: writer completes, readers see updated state, no panics/deadlocks

**Expected Result:** Test passes; concurrency is safe.

**Priority:** P0 (Critical)

**Grounding:** INV-6, STRATEGY.md lines 163-218

---

#### QA-2: Large State Persistence

**Description:** Large state (10,000+ accounts with storage and code) persists correctly.

**Verification:** Manual stress test or property test

**Test Steps:**
1. Generate 10,000 accounts with random balances, storage, and code
2. Call `with_genesis` or commit via `BundleState`
3. Close and reopen database
4. Sample 100 random accounts and verify data integrity

**Expected Result:** All sampled accounts match expected values; no corruption.

**Priority:** P1 (High)

**Grounding:** Performance risk from BLOCKERS.md BLK-103

---

#### QA-3: Empty BundleState Commit

**Description:** Committing an empty `BundleState` (no changes) is a no-op and does not cause errors.

**Verification:** Unit test

**Test Steps:**
1. Create empty `BundleState`
2. Call `commit(empty_bundle)`
3. Assert: returns `Ok(())`
4. Verify no MDBX write transaction is created (or it's a no-op)

**Expected Result:** Test passes; no error, no side effects.

**Priority:** P2 (Medium)

---

### Acceptance Criteria Summary Table

| ID | Description | Verification | Priority | Type |
|---|---|---|---|---|
| AC-1 | state-reth crate compiles | `cargo build -p state-reth` | P0 | Build |
| AC-2 | state-reth unit tests pass | `cargo test -p state-reth` | P0 | Test |
| AC-3 | state crate compiles (trait migration) | `cargo test -p state` | P0 | Build |
| AC-4 | state-memory adapts to fallible trait | `cargo test -p state-memory` | P0 | Build |
| AC-5 | app-evm consumer compiles | `cargo test -p app-evm` | P0 | Build |
| AC-6 | rpc-eth consumer compiles | `cargo test -p rpc-eth` | P0 | Build |
| AC-7 | whirlpool-node wiring compiles | `cargo test -p whirlpool-node` | P0 | Build |
| AC-8 | State persists across restarts | Test TC-SR-I001 | P0 | Integration |
| AC-9 | Genesis bootstrap correct | Tests TC-SR-I005, TC-SR-I006, TC-WN-I002 | P0 | Integration |
| AC-10 | BundleState commit works | Tests TC-SR-U003, TC-SR-U005, TC-SR-U007, TC-SR-I001 | P0 | Integration |
| AC-11 | Full workspace build succeeds | `cargo build` | P0 | Build |
| AC-12 | Full workspace tests pass | `cargo test` | P0 | Test |
| QA-1 | Concurrent reads during write safe | Test TC-SR-I004 | P0 | QA |
| QA-2 | Large state (10K+ accounts) persists | Manual/property test | P1 | QA |
| QA-3 | Empty BundleState commit is no-op | Unit test | P2 | QA |

**Total Criteria:** 12 acceptance criteria (P0) + 3 QA scenarios (P0-P2)

**Feature Gate:** ALL P0 criteria (AC-1 through AC-12, QA-1) MUST pass before feature merge.

## S4: Dependency Contract

This section documents the inter-crate dependency contracts, version constraints, feature flags, and breaking change cascades for the persistent state feature.

### Inter-Crate Dependency Contracts

#### state-reth Dependencies

**Workspace Dependencies:**

| Dependency | Type | Contract | Breaking Changes |
|---|---|---|---|
| `state` | Required | Implements `StateDb` trait; must satisfy associated `Error` type and `Result` return signatures | Breaking: trait signature changes cascade to state-reth |
| `state-memory` | None | No direct dependency; used only for behavioral comparison | N/A |

**External/Vendored Dependencies:**

| Dependency | Type | Path | Features | Contract |
|---|---|---|---|---|
| `reth-db` | Required | `vendor/reth/crates/storage/db` | `mdbx` | MDBX environment, transactions, table APIs |
| `reth-db-api` | Required | `vendor/reth/crates/storage/db-api` | default | Table trait definitions (`DbTx`, `DbTxMut`, `Table`) |
| `reth-storage-errors` | Required | `vendor/reth/crates/storage/errors` | default | `DatabaseError` type for error mapping |
| `reth-trie` | Required | `vendor/reth/crates/trie` | default | `StateRoot::overlay_root` for trie-based state root |
| `reth-codecs` | Required | `vendor/reth/crates/storage/codecs` | default | `Compact` codec used by reth storage tables |
| `revm` | Required | External | `std` | `Database` + `DatabaseRef` traits for EVM execution integration |
| `alloy-primitives` | Required | External | `std` | Shared primitive types (`Address`, `B256`, `U256`) |
| `thiserror` | Required | External | default | Error type derivation for `RethStateError` |

**Grounding:** STRATEGY.md lines 287-299, CRATES.md lines 29-42

---

#### whirlpool-node Dependencies

**Workspace Dependencies:**

| Dependency | Type | Contract | Breaking Changes |
|---|---|---|---|
| `state` | Existing | Generic trait bound for state backend | Breaking: trait signature changes require consumer updates |
| `state-reth` | **NEW** | Concrete persistent backend; replaces `state-memory` in production path | Breaking: initialization API changes require wiring updates |
| `state-memory` | Optional | Retained for testing/fallback only (can be removed if unused) | Non-breaking: removal is safe |
| `app-evm` | Existing | EVM execution context; generic over `StateProvider` | Non-breaking: wiring only |
| `rpc-eth` | Existing | RPC handler context; generic over `StateDb` | Non-breaking: wiring only |

**Grounding:** STRATEGY.md lines 301-306, CRATES.md lines 146-156

---

#### state-memory Dependencies

**Workspace Dependencies:**

| Dependency | Type | Contract | Breaking Changes |
|---|---|---|---|
| `state` | Existing | Implements `StateDb` trait with `type Error = Infallible` | Breaking: trait signature changes cascade to state-memory |

**Grounding:** STRATEGY.md lines 116-117, CRATES.md line 31

---

#### app-evm Dependencies

**Workspace Dependencies:**

| Dependency | Type | Contract | Breaking Changes |
|---|---|---|---|
| `state` | Existing | Generic trait bound (`impl StateDb`); no direct dependency on concrete implementations | Breaking: trait signature changes require error propagation updates |

**Grounding:** INTENT.md line 12, CRATES.md line 32

---

#### rpc-eth Dependencies

**Workspace Dependencies:**

| Dependency | Type | Contract | Breaking Changes |
|---|---|---|---|
| `state` | Existing | Generic trait bound (`impl StateDb`); no direct dependency on concrete implementations | Breaking: trait signature changes require error mapping updates |

**Grounding:** INTENT.md line 12, CRATES.md line 33

---

### Version Constraints and Feature Flags

#### state-reth Feature Flags

**Proposed:**

```toml
[features]
default = ["mdbx"]
mdbx = ["reth-db/mdbx"]
```

**Contract:**
- `mdbx` is the default feature for persistent backend
- Disabling `default` features is unsupported for production node runtime
- Feature surface is minimal; broader `reth-provider` stack is NOT included

**Grounding:** WORKSPACE.md lines 83-97

---

#### Vendored reth-db Version Pin

**Current Strategy:** Use vendored `reth` monorepo at `vendor/reth/` (commit hash pinned by workspace).

**Upgrade Policy:**
- reth upgrades require validation of table APIs, codec contracts, and trie semantics
- Breaking changes in reth storage stack cascade to `state-reth` implementation

**Risk:** reth-db encoding may diverge from revm types → mitigated by explicit `codec.rs` module

**Grounding:** STRATEGY.md lines 56-61

---

#### revm Version Constraint

**Current Version:** `revm = "34"` (exact version TBD during implementation)

**Contract:**
- `revm::Database` and `revm::DatabaseRef` traits must remain stable
- Error type must implement `revm::primitives::DBError`

**Breaking Change Cascade:**
- revm trait signature changes require updates to `state::StateDb` trait and all implementations

**Grounding:** STRATEGY.md line 296, PC-007

---

### Breaking Change Cascades

#### Cascade 1: StateDb Trait Signature Change

**Trigger:** Modification to `state::StateDb` trait (new methods, signature changes, trait bounds)

**Affected Crates:**
1. `state-memory` — must update implementation
2. `state-reth` — must update implementation
3. `app-evm` — may require error propagation updates
4. `rpc-eth` — may require error mapping updates
5. `whirlpool-node` — may require wiring updates

**Mitigation:** Trait changes are controlled by `state` crate; coordination required before breaking changes

**Grounding:** BLOCKERS.md BLK-001, CRATES.md lines 124-137

---

#### Cascade 2: reth-db Table API Change

**Trigger:** Vendored reth-db updates change table encoding, transaction APIs, or trie semantics

**Affected Crates:**
1. `state-reth` — table access layer (`tables.rs`, `codec.rs`) must update
2. `whirlpool-node` — may require re-initialization of MDBX databases (migration)

**Mitigation:** Pin reth version; validate upgrades with integration tests before merging

**Grounding:** STRATEGY.md lines 287-299

---

#### Cascade 3: revm Database Trait Change

**Trigger:** revm updates change `Database` or `DatabaseRef` trait signatures

**Affected Crates:**
1. `state` — trait design may need alignment
2. `state-reth` — implementation must update
3. `state-memory` — implementation must update
4. `app-evm` — execution path may require updates

**Mitigation:** Pin revm version; coordinate upgrades with trait design

**Grounding:** STRATEGY.md lines 86-87, PC-007

---

### Dependency Graph (Build Order)

```text
Tier 1 (Interface):
  state (trait authority)
    ├─> alloy-primitives
    └─> revm (trait integration)

Tier 2 (Implementations):
  state-memory (reference impl)
    └─> state

  state-reth (persistent impl)
    ├─> state
    ├─> reth-db (+ mdbx feature)
    ├─> reth-db-api
    ├─> reth-storage-errors
    ├─> reth-trie
    ├─> reth-codecs
    ├─> revm
    └─> alloy-primitives

Tier 3 (Consumers):
  app-evm (generic consumer)
    └─> state

  rpc-eth (generic consumer)
    └─> state

Tier 4 (Wiring):
  whirlpool-node (runtime composition)
    ├─> state
    ├─> state-reth
    ├─> app-evm
    └─> rpc-eth
```

**Compilation Order:** Tier 1 → Tier 2 → Tier 3 → Tier 4

**Grounding:** WORKSPACE.md lines 30-60

---

### Dependency Contract Summary Table

| Crate | Direct Deps | Breaking Change Sources | Coordination Required |
|---|---|---|---|
| `state` | revm, alloy-primitives | revm trait changes | Yes (trait authority) |
| `state-memory` | state | state trait changes | No (implementation follows trait) |
| `state-reth` | state, reth-db stack, revm | state trait, reth-db API, revm traits | Yes (bridges multiple upstream sources) |
| `app-evm` | state | state trait changes | No (generic consumer) |
| `rpc-eth` | state | state trait changes | No (generic consumer) |
| `whirlpool-node` | state, state-reth, app-evm, rpc-eth | state trait, state-reth init API | Yes (wiring coordinator) |

**High-Coordination Crates:** `state`, `state-reth`, `whirlpool-node`

## S5: Risk Assessment

This section identifies residual risks that remain after design completion, along with their likelihood, impact, and mitigation strategies. All risks are tied to specific implementation concerns.

### Risk Classification

**Likelihood:** Low / Medium / High
**Impact:** Low / Medium / High / Critical
**Status:** Open / Mitigated / Accepted

---

### R-1: MDBX Build Failures in CI (Host Prerequisites)

**Description:** MDBX builds via `mdbx-sys` FFI bindings require a C compiler (clang) and bindgen toolchain. If the CI environment or developer shell lacks these prerequisites, `cargo build -p state-reth` will fail.

**Likelihood:** Medium (depends on environment setup)

**Impact:** Critical (blocks feature build and testing)

**Affected Components:**
- `state-reth` crate compilation
- CI pipeline (build + test stages)
- Developer onboarding (local builds)

**Mitigation Strategy:**
1. **Immediate:** Verify `flake.nix` includes C compiler + clang + bindgen dependencies
2. **Verification:** Run `nix develop --command clang --version` during Phase 1 (pre-condition PC-005)
3. **Documentation:** Add explicit build prerequisites to `state-reth/README.md` and workspace docs
4. **CI:** Ensure Nix dev shell is used for all CI build steps
5. **Fallback:** If Nix shell is incomplete, update `flake.nix` to include missing packages

**Grounding:** BLOCKERS.md BLK-003, PC-005 (assumption flag), SUMMARY.md Risk 5

**Status:** Open (requires Phase 1 verification)

---

### R-2: reth-db Type Encoding Divergence

**Description:** reth-db uses custom `Compact` codecs for storage encoding. If reth storage stack updates change encoding semantics or table schemas, the `codec.rs` translation layer may produce incorrect conversions between revm types (`AccountInfo`, `Bytecode`) and reth types, leading to silent data corruption.

**Likelihood:** Low (vendored reth is pinned; upgrades are controlled)

**Impact:** High (silent data corruption, incorrect state roots)

**Affected Components:**
- `state-reth::codec.rs` (type translation)
- `state-reth::tables.rs` (table access)
- State root computation (trie correctness)

**Mitigation Strategy:**
1. **Design:** Explicit `codec.rs` module isolates translation logic
2. **Testing:** Property tests for round-trip codec correctness (TC-SR-U016, TC-SR-U017)
3. **Upgrade Protocol:** Validate codec behavior with integration tests before merging reth updates
4. **Monitoring:** Add state root sanity checks (compare against known fixtures)

**Grounding:** STRATEGY.md lines 56-61, lines 278-282

**Status:** Mitigated (isolated in codec module + property tests)

---

### R-3: Per-Method Transaction Performance Bottleneck

**Description:** The concurrency model uses per-method transaction acquisition (short-lived read/write tx). Under high RPC load or concurrent EVM execution, frequent transaction creation/teardown may introduce performance overhead, especially for read-heavy workloads.

**Likelihood:** Medium (depends on workload profile)

**Impact:** Medium (performance degradation, not correctness issue)

**Affected Components:**
- `state-reth::db.rs` (transaction lifecycle)
- RPC handlers (high-frequency reads)
- EVM execution (mixed read/write)

**Mitigation Strategy:**
1. **Phase 1 Design:** Start with correctness-first per-method transactions (simplifies concurrency)
2. **Measurement:** Add performance benchmarks (reads/sec, writes/sec, tx overhead)
3. **Deferred Optimization:** If profiling shows bottleneck, add caching layer or transaction pooling
4. **Documentation:** Mark optimization as future work (BLOCKERS.md BLK-103)

**Grounding:** BLOCKERS.md BLK-103, STRATEGY.md lines 176-218

**Status:** Accepted (deferred to post-implementation profiling)

---

### R-4: Fallible StateDb Breaking Change Cascade

**Description:** Making `StateDb` trait fallible (adding `type Error` + `Result` returns) is a breaking change that cascades to ALL implementations and consumers. If error propagation is not handled correctly in `app-evm` or `rpc-eth`, it may introduce panics or incorrect error responses.

**Likelihood:** High (trait change affects all crates)

**Impact:** High (blocks compilation, requires coordinated updates)

**Affected Components:**
- `state` crate (trait definition)
- `state-memory` (infallible impl)
- `state-reth` (fallible impl)
- `app-evm` (error propagation in execution path)
- `rpc-eth` (error mapping to JSON-RPC errors)
- `whirlpool-node` (wiring error handling)

**Mitigation Strategy:**
1. **Design:** Use `type Error = Infallible` for `state-memory` (minimal code changes)
2. **Phased Migration:** Update trait → update implementations → update consumers (build order)
3. **Testing:** Compile checks (AC-3 through AC-7) ensure all crates build
4. **Error Handling:** Leverage existing `revm::Database::Error` propagation patterns in `app-evm`
5. **Documentation:** Clear migration guide in `state/README.md`

**Grounding:** BLOCKERS.md BLK-001, STRATEGY.md lines 81-121, INV-7

**Status:** Mitigated (phased migration + existing error handling patterns)

---

### R-5: Genesis Initialization Race Condition

**Description:** If multiple node processes attempt to initialize the same MDBX database simultaneously (e.g., in a clustered deployment), race conditions may occur during genesis bootstrap, leading to duplicate writes or lock contention.

**Likelihood:** Low (single-node deployment expected; multi-node requires coordination)

**Impact:** Medium (node startup failure, non-deterministic behavior)

**Affected Components:**
- `whirlpool-node` genesis initialization logic
- `state-reth::init.rs` (database creation)
- MDBX write transaction locking

**Mitigation Strategy:**
1. **Phase 1:** Assume single-node deployment (no multi-process coordination)
2. **Detection:** Add empty-DB check before genesis initialization (TC-WN-I002)
3. **Future:** If multi-node deployment required, add filesystem locking or coordination service
4. **Documentation:** Note single-node assumption in `whirlpool-node/README.md`

**Grounding:** FLOWS.md section 2 (Genesis Bootstrap Flow), TESTS.md Q5

**Status:** Accepted (single-node assumption; multi-node is future work)

---

### R-6: Trie Root Semantic Divergence from state-memory

**Description:** `state-reth` uses reth's trie-based state root (`StateRoot::overlay_root`), while `state-memory` uses keccak256 hash over sorted state. This means the same logical state produces DIFFERENT roots depending on the backend. Tests written against `state-memory` baseline will fail if they expect identical roots.

**Likelihood:** High (by design — different root computation methods)

**Impact:** Medium (test compatibility, not functional issue)

**Affected Components:**
- State root computation tests
- Test fixtures and baselines
- Documentation (state root semantics)

**Mitigation Strategy:**
1. **Design:** Explicit documentation of semantic divergence in `state-reth/README.md`
2. **Testing:** Separate test fixtures for trie-backed correctness (TC-SR-U009, TC-SR-U010)
3. **Validation:** Use Ethereum test vectors for trie root correctness (not in-memory baseline)
4. **Communication:** Note divergence in migration guide and design docs

**Grounding:** STRATEGY.md lines 220-256, BLOCKERS.md BLK-002

**Status:** Accepted (intentional design choice; documented)

---

### R-7: MDBX Database Corruption on Ungraceful Shutdown

**Description:** If the node process crashes or is killed (SIGKILL) during a write transaction, MDBX may leave the database in an inconsistent state. Recovery requires rollback to last committed transaction.

**Likelihood:** Low (MDBX has ACID guarantees, but OS-level crashes can corrupt)

**Impact:** High (node restart failure, data loss)

**Affected Components:**
- MDBX database durability
- Node shutdown handling
- Recovery/restart logic

**Mitigation Strategy:**
1. **Design:** MDBX transactions are atomic; uncommitted writes are rolled back on process exit
2. **Graceful Shutdown:** Implement clean shutdown handler in `whirlpool-node` (TC-WN-I004)
3. **Recovery:** Test DB reopen after ungraceful shutdown (verify MDBX recovery)
4. **Monitoring:** Add startup health check (verify DB integrity on open)

**Grounding:** INV-5 (commit atomicity), STRATEGY.md lines 194-203

**Status:** Mitigated (MDBX ACID guarantees + graceful shutdown)

---

### R-8: Block Hash Table Mapping Ambiguity (Soft Blocker)

**Description:** The exact reth-db table for `get_block_hash` / `insert_block_hash` is not yet pinned (`CanonicalHeaders` vs `HeaderNumbers`). Implementation may choose wrong table, requiring rework.

**Likelihood:** Low (implementation-time decision, not fundamental risk)

**Impact:** Low (rework table access layer, no breaking changes)

**Affected Components:**
- `state-reth::tables.rs` (block hash table access)
- `StateDb::get_block_hash` / `insert_block_hash` implementation

**Mitigation Strategy:**
1. **Phase 2:** Explore reth-db API during implementation to confirm table choice
2. **Fallback:** If ambiguous, prioritize `CanonicalHeaders` (matches reth naming conventions)
3. **Testing:** Validate block hash persistence in integration tests (TC-SR-U008)

**Grounding:** BLOCKERS.md BLK-101, STRATEGY.md lines 271-273

**Status:** Accepted (soft blocker, resolved during implementation)

---

### Risk Assessment Summary Table

| ID | Risk | Likelihood | Impact | Status | Mitigation Owner |
|---|---|---|---|---|---|
| R-1 | MDBX build fails (host prerequisites) | Medium | Critical | Open | Phase 1 verification |
| R-2 | reth-db encoding divergence | Low | High | Mitigated | Codec module + property tests |
| R-3 | Per-method tx performance | Medium | Medium | Accepted | Deferred profiling |
| R-4 | Fallible StateDb breaking change | High | High | Mitigated | Phased migration |
| R-5 | Genesis race condition | Low | Medium | Accepted | Single-node assumption |
| R-6 | Trie root semantic divergence | High | Medium | Accepted | Documented design choice |
| R-7 | MDBX corruption on ungraceful shutdown | Low | High | Mitigated | MDBX ACID + graceful shutdown |
| R-8 | Block hash table mapping TBD | Low | Low | Accepted | Implementation-time decision |

**Critical Risks:** R-1 (requires immediate verification)

**Open Risks:** R-1 (MDBX prerequisites)

**Accepted Risks:** R-3, R-5, R-6, R-8 (by design or deferred)

**Mitigated Risks:** R-2, R-4, R-7 (design + testing strategies in place)
