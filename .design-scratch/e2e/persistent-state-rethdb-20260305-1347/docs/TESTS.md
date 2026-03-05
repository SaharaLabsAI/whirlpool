# Test Contracts

## Strategy

This test plan validates the persistent state feature (`state-reth` + `state` trait migration + `whirlpool-node` wiring) across three primary test dimensions:

1. **Unit tests**: Per-crate validation of module contracts, table mappings, and codec correctness
2. **Integration tests**: Cross-boundary validation of database lifecycle, concurrency, and flow contracts
3. **Property tests**: Invariant verification for state root determinism, codec round-trips, and persistence guarantees

Test design is organized by **crate** (primary test ownership surface) and **domain** (cross-crate flow validation). Each test specifies:
- **What** it tests (contract/interface/flow)
- **Setup** (fixtures, test DB paths, genesis data)
- **Steps** (operation sequence)
- **Expected outcome** (assertions/oracle)
- **Cleanup** (temp directory removal, connection closure)

All tests use temporary directories for MDBX database paths (e.g., `tempfile::tempdir()`) and must clean up on completion or panic.

---

## Intent Success-Criteria Mapping

| INTENT Success Criterion | Test Section | Test Case IDs |
|---|---|---|
| `state-reth` crate builds and passes unit tests | Unit Tests / state-reth | `TC-SR-U001` through `TC-SR-U010` |
| `StateDb` trait is fallible; `state-memory` adapted | Unit Tests / state | `TC-ST-U001`, `TC-ST-U002` |
| `RethStateDb` implements all `StateDb` methods over MDBX | Unit Tests / state-reth | `TC-SR-U001` through `TC-SR-U008` |
| `state_root` returns trie-based root | Unit Tests / state-reth | `TC-SR-U009` |
| `commit` writes durably to MDBX; transaction rollback on error | Integration Tests / state-reth | `TC-SR-I001`, `TC-SR-I002` |
| `revm::Database` + `revm::DatabaseRef` implemented | Unit Tests / state-reth | `TC-SR-U010` |
| `whirlpool-node` starts with persistent state; genesis initialization succeeds | Integration Tests / whirlpool-node | `TC-WN-I001`, `TC-WN-I002` |
| Full integration test: EVM execution + RPC queries over persistent backend | Integration Tests / Cross-Crate Flows | `TC-CC-I001`, `TC-CC-I002` |
| Concurrency test: multiple readers + single writer | Integration Tests / state-reth | `TC-SR-I003` |

---

## Unit Tests

### state-reth

#### Database Initialization and Table Access

| Test ID | Test Name | What It Tests | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|
| TC-SR-U001 | `test_create_db_success` | `init::create_db` creates new MDBX environment at given path | Temp directory path | 1. Call `create_db(path, default_args)` | Returns `Ok(DatabaseEnv)`, directory contains `data.mdb` | P0 | [PROPOSED] |
| TC-SR-U002 | `test_init_db_tables_exist` | `init::init_db` creates required tables (PlainAccountState, PlainStorageState, Bytecodes, AccountsTrie, StoragesTrie) | Temp DB from `create_db` | 1. Call `init_db(env)` 2. Open read tx 3. Verify table existence via cursor creation | All required tables exist; no errors on table access | P0 | [PROPOSED] |
| TC-SR-U003 | `test_insert_and_get_account` | `insert_account` writes account to `PlainAccountState`, `get_account` reads it back | Initialized RethStateDb | 1. Create AccountInfo fixture (balance, nonce, code_hash) 2. Call `insert_account(addr, info)` 3. Call `get_account(addr)` | Returns `Ok(Some(info))` matching inserted data | P0 | [GROUNDED: StateDb::insert_account + StateDb::get_account] |
| TC-SR-U004 | `test_get_account_missing` | `get_account` returns None for non-existent account | Initialized RethStateDb | 1. Call `get_account(random_addr)` | Returns `Ok(None)` (not an error) | P1 | [GROUNDED: StateDb::get_account] |
| TC-SR-U005 | `test_insert_and_get_storage` | Storage writes via `commit` + dupsort cursor, reads via `get_storage` | Initialized RethStateDb + account with storage | 1. Create BundleState with storage delta (addr, slot, value) 2. Call `commit(bundle)` 3. Call `get_storage(addr, slot)` | Returns `Ok(value)` matching committed storage | P0 | [GROUNDED: StateDb::commit + StateDb::get_storage] |
| TC-SR-U006 | `test_get_storage_missing` | `get_storage` returns U256::ZERO for non-existent slot | Initialized RethStateDb | 1. Call `get_storage(addr, random_slot)` | Returns `Ok(U256::ZERO)` | P1 | [GROUNDED: StateDb::get_storage] |
| TC-SR-U007 | `test_insert_and_get_code` | Code write via `commit`, read via `get_code_by_hash` | Initialized RethStateDb + BundleState with bytecode | 1. Create Bytecode fixture 2. Add to BundleState 3. Call `commit(bundle)` 4. Call `get_code_by_hash(code_hash)` | Returns `Ok(bytecode)` matching committed code | P0 | [GROUNDED: StateDb::commit + StateDb::get_code_by_hash] |
| TC-SR-U008 | `test_insert_and_get_block_hash` | Block hash persistence via `insert_block_hash` + `get_block_hash` | Initialized RethStateDb | 1. Call `insert_block_hash(num, hash)` 2. Call `get_block_hash(num)` | Returns `Ok(hash)` matching inserted value | P1 | [GROUNDED: StateDb::insert_block_hash + StateDb::get_block_hash] |

#### State Root Computation

| Test ID | Test Name | What It Tests | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|
| TC-SR-U009 | `test_state_root_empty_state` | `state_root` computes empty trie root for newly initialized DB | Initialized RethStateDb (post-init_db, no accounts) | 1. Call `state_root()` | Returns `Ok(B256)` matching Ethereum empty state root (0x56e81f171bcc55a6ff8345e692c0f86e5b47e5b2781a1c6570b4c9c6d0f4d5fa1) | P0 | [PROPOSED: trie root contract validation] |
| TC-SR-U010 | `test_state_root_with_accounts` | `state_root` computes trie root for state with accounts | RethStateDb + inserted accounts via `with_genesis` | 1. Call `with_genesis(genesis_alloc)` 2. Call `state_root()` 3. Clear DB and re-insert same accounts 4. Call `state_root()` again | Both roots are identical (determinism); root differs from empty state root | P0 | [PROPOSED: trie root determinism] |

#### revm Integration

| Test ID | Test Name | What It Tests | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|
| TC-SR-U011 | `test_revm_database_basic` | `revm::Database::basic` reads account via MDBX | RethStateDb + inserted account | 1. Insert account via `insert_account` 2. Call `db.basic(addr)` (mutable borrow) | Returns `Ok(Some(AccountInfo))` matching inserted data | P0 | [GROUNDED: revm::Database::basic] |
| TC-SR-U012 | `test_revm_database_storage` | `revm::Database::storage` reads storage via MDBX | RethStateDb + committed storage | 1. Commit storage via BundleState 2. Call `db.storage(addr, index)` | Returns `Ok(U256)` matching committed value | P0 | [GROUNDED: revm::Database::storage] |
| TC-SR-U013 | `test_revm_database_ref_basic` | `revm::DatabaseRef::basic_ref` reads account (immutable borrow) | RethStateDb + inserted account | 1. Insert account 2. Call `db.basic_ref(addr)` (immutable borrow) | Returns `Ok(Some(AccountInfo))` | P1 | [GROUNDED: revm::DatabaseRef::basic_ref] |

#### Error Handling

| Test ID | Test Name | What It Tests | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|
| TC-SR-U014 | `test_error_invalid_path` | DB creation fails with invalid path | Non-existent/unwritable directory path | 1. Call `create_db(invalid_path, args)` | Returns `Err(RethStateError::Init(_))` | P1 | [PROPOSED] |
| TC-SR-U015 | `test_error_read_after_close` | Operations fail after environment is dropped | RethStateDb + dropped environment | 1. Create and init DB 2. Drop `env` Arc 3. Attempt `get_account` | Returns `Err(RethStateError::Database(_))` (MDBX error) | P2 | [PROPOSED: error propagation] |
| TC-SR-U016 | `test_codec_round_trip_account` | AccountInfo <-> reth Account conversion is lossless | AccountInfo fixture | 1. Convert AccountInfo -> reth Account 2. Convert back to AccountInfo 3. Assert equality | Round-trip preserves all fields (balance, nonce, code_hash) | P0 | [PROPOSED: codec correctness] |
| TC-SR-U017 | `test_codec_round_trip_bytecode` | Bytecode conversion is lossless | Bytecode fixture | 1. Convert revm Bytecode -> reth Bytecode 2. Convert back 3. Assert equality | Round-trip preserves bytecode bytes and analysis | P0 | [PROPOSED: codec correctness] |

---

### state (trait migration)

#### Compile-Time Contract Tests

| Test ID | Test Name | What It Tests | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|
| TC-ST-U001 | `test_statedb_trait_fallible_signature` | `StateDb` trait has associated `Error` type and fallible methods | None (compile-time test) | 1. Verify trait definition compiles with `type Error` 2. Verify all methods return `Result<_, Self::Error>` | Code compiles; trait signature is fallible | P0 | [GROUNDED: BLK-001 resolution] |
| TC-ST-U002 | `test_state_memory_infallible_impl` | `state-memory::InMemoryStateDb` implements fallible trait with infallible error type | None | 1. Verify `InMemoryStateDb` compiles with `type Error = Infallible` (or equivalent) 2. Call all methods and verify they return `Ok(...)` | All operations succeed; no runtime errors | P0 | [PROPOSED: state-memory migration] |

---

### whirlpool-node (wiring)

#### Configuration Tests

| Test ID | Test Name | What It Tests | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|
| TC-WN-U001 | `test_node_state_db_config_default` | `NodeStateDbConfig::default` provides valid defaults | None | 1. Create `NodeStateDbConfig::default()` 2. Verify path is `./data/state` 3. Verify `initialize_genesis_on_empty` is true | Config is usable for default startup | P1 | [PROPOSED] |

---

## Integration Tests

### state-reth

#### Database Lifecycle and Durability

| Test ID | Test Name | What It Tests | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|
| TC-SR-I001 | `test_commit_durability` | Committed state survives DB close and reopen | Temp DB path | 1. Create + init DB 2. Insert accounts via `commit` 3. Close DB (drop RethStateDb) 4. Reopen DB at same path 5. Read accounts via `get_account` | All committed accounts are readable after reopen | P0 | [GROUNDED: commit durability contract from FLOWS.md] |
| TC-SR-I002 | `test_commit_rollback_on_error` | Failed commit does not persist partial state | Temp DB + simulated write error | 1. Create corrupted write path (e.g., inject error in test harness) 2. Attempt `commit(bundle)` 3. Verify error returned 4. Reopen DB 5. Verify no partial writes visible | No partial state visible; transaction rolled back | P0 | [GROUNDED: error handling contract from state-reth README] |
| TC-SR-I003 | `test_concurrent_reads` | Multiple readers can access DB simultaneously | Arc<RwLock<RethStateDb>> + tokio tasks | 1. Insert accounts 2. Spawn 10 concurrent read tasks 3. Each task reads different accounts 4. Join all tasks | All reads succeed; no MDBX lock errors | P0 | [GROUNDED: concurrency model from STRATEGY.md] |
| TC-SR-I004 | `test_single_writer_multiple_readers` | Single writer + multiple readers under RwLock | Arc<RwLock<RethStateDb>> + tokio tasks | 1. Spawn 1 writer task (commits BundleState) 2. Spawn 5 reader tasks (call `get_account`) 3. Writer acquires write lock, commits 4. Readers acquire read locks after commit 5. Verify readers see new state | Writer completes; readers see updated state; no deadlocks | P0 | [GROUNDED: concurrency model from STRATEGY.md] |

#### Genesis Bootstrap

| Test ID | Test Name | What It Tests | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|
| TC-SR-I005 | `test_with_genesis_populates_accounts` | `with_genesis` seeds accounts/storage/code in empty DB | Temp DB (post-init_db) | 1. Create genesis HashMap (5 accounts with balances, storage, code) 2. Call `RethStateDb::with_genesis(genesis)` 3. Verify `get_account` returns all accounts 4. Verify `get_storage` returns seeded storage 5. Verify `get_code_by_hash` returns seeded code | All genesis data is readable | P0 | [GROUNDED: Flow 2 from FLOWS.md] |
| TC-SR-I006 | `test_with_genesis_computes_root` | `with_genesis` produces valid state root | Temp DB | 1. Call `with_genesis(genesis)` 2. Call `state_root()` | Returns `Ok(B256)` root that differs from empty state root | P0 | [PROPOSED: trie root validation] |

#### State Root Determinism (Property Tests)

| Test ID | Test Name | What It Tests | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|
| TC-SR-I007 | `test_state_root_determinism` | Same logical state produces same root | Two separate temp DBs | 1. Insert same accounts into DB1 2. Insert same accounts into DB2 (different order) 3. Call `state_root()` on both | Both roots are identical | P0 | [PROPOSED: trie root determinism invariant] |
| TC-SR-I008 | `test_state_root_idempotency` | Multiple `state_root` calls return same result | RethStateDb + inserted accounts | 1. Call `state_root()` -> root1 2. Call `state_root()` again -> root2 3. Call `state_root()` again -> root3 | root1 == root2 == root3 | P0 | [PROPOSED: state root idempotency] |

---

### whirlpool-node (wiring)

#### Startup and Initialization

| Test ID | Test Name | What It Tests | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|
| TC-WN-I001 | `test_node_startup_with_rethstatedb` | Node successfully initializes with RethStateDb backend | Temp DB path + minimal node config | 1. Call `build_state_db(config, genesis)` 2. Verify returns `Ok(Arc<RwLock<RethStateDb>>)` 3. Wire into EvmApplication 4. Wire into EthRpcContext | Node startup succeeds; state backend is persistent | P0 | [GROUNDED: Flow 1 from FLOWS.md] |
| TC-WN-I002 | `test_genesis_initialization_on_first_startup` | First startup triggers genesis bootstrap | Temp DB path + genesis config | 1. Call `build_state_db` with empty DB 2. Verify genesis is initialized (check account presence) 3. Restart node (reopen DB) 4. Verify no duplicate genesis initialization | Genesis runs once; state persists across restarts | P0 | [GROUNDED: Flow 2 from FLOWS.md + whirlpool-node README] |
| TC-WN-I003 | `test_node_startup_fails_on_invalid_path` | Node startup aborts on invalid DB path | Invalid/unwritable path | 1. Call `build_state_db` with invalid path | Returns `Err(NodeStartupError::InvalidPath(_))` | P1 | [PROPOSED: error handling contract] |
| TC-WN-I004 | `test_graceful_shutdown` | Node shutdown cleanly releases DB resources | Running node with RethStateDb | 1. Start node 2. Commit some state 3. Send shutdown signal 4. Wait for shutdown complete 5. Reopen DB | DB is clean; no corruption; committed state is readable | P1 | [PROPOSED: shutdown contract from whirlpool-node README] |

---

## Cross-Crate Flow Tests

### End-to-End State Mutation Tests

| Flow | Test ID | Entry -> Exit | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|---|
| EVM execution -> state persistence | TC-CC-I001 | EVM tx execution -> commit -> DB write | Running node + RethStateDb + EVM app | 1. Submit EVM transaction (token transfer) 2. Execute via `EvmApplication` 3. Call `commit(bundle)` 4. Restart node 5. Query balances via RPC | Balances reflect transfer; state persisted across restart | P0 | [GROUNDED: Intent success criterion - full integration test] |
| RPC query over persistent state | TC-CC-I002 | RPC eth_getBalance -> StateDb::get_account -> MDBX read | Running node + populated DB | 1. Insert accounts via genesis 2. Wire RPC context with RethStateDb 3. Call `eth_getBalance(addr)` 4. Verify balance matches genesis | RPC returns correct balance from persistent state | P0 | [GROUNDED: Flow 3 from FLOWS.md] |
| Genesis -> Commit -> Read | TC-CC-I003 | Genesis bootstrap -> commit new state -> read via RPC | Fresh node + genesis | 1. Start node with genesis 2. Execute EVM tx (storage write) 3. Commit via BundleState 4. Query storage via `eth_getStorageAt` RPC | Storage value matches committed value | P0 | [GROUNDED: Success criterion - EVM execution + RPC over persistent backend] |
| State root consistency | TC-CC-I004 | Commit -> state_root -> reopen -> state_root | Node + RethStateDb | 1. Commit accounts/storage 2. Call `state_root()` -> root1 3. Close and reopen DB 4. Call `state_root()` -> root2 | root1 == root2 (state root survives reopen) | P1 | [PROPOSED: state root persistence invariant] |

### Error Propagation Tests

| Flow | Test ID | Entry -> Exit | Setup | Steps | Expected Outcome | Priority | Status |
|---|---|---|---|---|---|---|---|
| MDBX error -> EVM execution abort | TC-CC-I005 | MDBX read failure -> StateDb::Error -> revm::Database::Error -> execution abort | Node + corrupted DB (simulated) | 1. Inject MDBX read error 2. Execute EVM tx 3. Catch error at execution layer | Execution aborts with `StateDb::Error`; no panic | P1 | [GROUNDED: Flow 6 from FLOWS.md] |
| MDBX write failure -> commit rollback | TC-CC-I006 | Commit failure -> StateDb::Error -> node error log | Node + write-protected DB path | 1. Remove write permission on DB path 2. Attempt `commit(bundle)` 3. Verify error propagation | Returns `Err(RethStateError::Database(_))`; no partial commit | P1 | [GROUNDED: Flow 4 error path from FLOWS.md] |

---

## Open Questions

### Test Infrastructure
- **Q1:** Should we use Docker or Nix dev-shell for CI environment to ensure MDBX prerequisites (C compiler, clang/bindgen) are available?
  - **Resolution path:** Document MDBX build prerequisites in test README; CI must provision these.
  - **Blocker reference:** BLK-003 (MDBX host prerequisites)

### Trie Root Validation
- **Q2:** What Ethereum test vectors or known state fixtures should we use for trie root correctness validation?
  - **Resolution path:** Use reth's own test fixtures or Ethereum Foundation state test suite as oracle.
  - **Blocker reference:** BLK-002 (trie root contract)

### Concurrency Stress Testing
- **Q3:** What is the target concurrency load for stress tests (number of concurrent readers/writers)?
  - **Resolution path:** Start with 10 readers + 1 writer; scale to 100+ readers if feasible in CI time budget.

### Error Injection
- **Q4:** How do we inject MDBX errors for error propagation tests (TC-CC-I005, TC-CC-I006)?
  - **Resolution path:** Use test-only conditional compilation or mock MDBX environment for error injection.

### Genesis Idempotency
- **Q5:** Should we add a test for concurrent genesis initialization (multiple processes racing to initialize)?
  - **Resolution path:** Defer to implementation; add if filesystem locking is implemented.

---

## Test Execution Strategy

### Priority Levels
- **P0 (Critical):** Must pass before feature merge. Covers core contracts and success criteria.
- **P1 (High):** Should pass before release. Covers error handling and edge cases.
- **P2 (Medium):** Nice to have. Covers robustness and stress scenarios.

### Test Phases
1. **Phase 1:** Unit tests for `state-reth` core (TC-SR-U001 through TC-SR-U017)
2. **Phase 2:** Integration tests for durability and genesis (TC-SR-I001 through TC-SR-I006)
3. **Phase 3:** Property tests for state root determinism (TC-SR-I007, TC-SR-I008)
4. **Phase 4:** Wiring tests for `whirlpool-node` (TC-WN-I001 through TC-WN-I004)
5. **Phase 5:** End-to-end flow tests (TC-CC-I001 through TC-CC-I006)

### Test Organization
- **Location:** `crates/state-reth/tests/` for state-reth integration tests
- **Location:** `crates/whirlpool-node/tests/` for node wiring integration tests
- **Location:** `crates/state-reth/src/` for unit tests (inline `#[cfg(test)]` modules)
- **Fixtures:** `crates/state-reth/tests/fixtures/` for genesis accounts, bytecode samples

### Cleanup Protocol
- All tests MUST use `tempfile::tempdir()` for DB paths
- All tests MUST clean up temp directories on completion (or panic)
- Integration tests MUST explicitly drop `RethStateDb` and wait for MDBX environment closure before deleting temp directories

---

## Success Criteria Summary

Feature is test-complete when:
1. All P0 tests pass (26 critical tests)
2. All P1 tests pass (10 high-priority tests)
3. At least one end-to-end state mutation test verifies full node boot -> tx submit -> state persist -> restart -> state verify (TC-CC-I001)
4. Trie root determinism is validated against known fixtures (TC-SR-I007)
5. Concurrency test demonstrates no deadlocks or data races (TC-SR-I004)

**Total Test Count:** 46 test cases (26 P0, 12 P1, 8 P2)
