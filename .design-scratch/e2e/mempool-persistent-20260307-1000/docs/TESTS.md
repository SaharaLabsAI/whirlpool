# Test Contracts

## Strategy

This test plan validates the persistent mempool implementation through unit tests (per-crate interface verification), integration tests (cross-crate flow validation), and crash recovery scenarios. Testing follows TDD principles with test-first hooks defined in FLOWS.md implementation slices.

**Testing Philosophy**:
- **Unit tests** verify individual components in isolation (MDBX operations, trait implementations)
- **Integration tests** validate end-to-end flows across crate boundaries (RPC → mempool → consensus)
- **Property tests** ensure invariants hold under concurrent access and edge cases
- **Crash recovery tests** validate ACID guarantees and data durability

**Test Dependencies**:
- `tempdir` crate for isolated MDBX database directories per test
- `tokio::test` for async test cases (RPC, consensus integration)
- Standard `#[test]` for synchronous unit tests

## Intent success-criteria mapping

| INTENT success criterion | Test section | Test case IDs |
|--------------------------|--------------|---------------|
| Transactions survive node restarts | Integration Tests (Crash Recovery) | INT-CR-01, INT-CR-02, INT-CR-03 |
| TxSource trait contract preserved | Unit Tests (app, mempool) | UT-APP-01, UT-MEMPOOL-04 |
| Push semantics functional | Unit Tests (mempool) | UT-MEMPOOL-01, UT-MEMPOOL-02, UT-MEMPOOL-05 |
| Drain semantics preserved (FIFO) | Unit Tests (mempool) | UT-MEMPOOL-03, UT-MEMPOOL-06 |
| Concurrent access safe | Unit Tests (mempool) | UT-MEMPOOL-07, UT-MEMPOOL-08 |
| Integration with whirlpool-node | Integration Tests (Node Wiring) | INT-WIRE-01, INT-WIRE-02 |
| EthRpcContext accepts trait object | Unit Tests (rpc-eth) | UT-RPC-01, UT-RPC-02 |

---

## Unit Tests

### app

| Interface | Test case ID | Path (happy/failure) | Setup | Stimulus | Assertions/Oracle | Priority | Status |
|-----------|--------------|----------------------|-------|----------|-------------------|----------|--------|
| `TxSource::push()` trait method exists | UT-APP-01 | Happy | Create `InMemoryTxPool`, cast to `&dyn TxSource` | Call `push(vec![0x01, 0x02])`, then `pending()` | Assert `pending()` returns 1 tx, trait object compiles | P0 | [PROPOSED] |
| `InMemoryTxPool` implements new trait signature | UT-APP-02 | Happy | Create `InMemoryTxPool` | Call `push()` via trait object, verify storage | Assert tx stored and retrievable via `pending()` | P0 | [PROPOSED] |
| `NoopTxSource::push()` is no-op | UT-APP-03 | Happy | Create `NoopTxSource` | Call `push(vec![0xAA])`, then `pending()` | Assert `pending()` returns empty vec | P1 | [PROPOSED] |
| Trait bounds `Send + Sync` satisfied | UT-APP-04 | Happy | Compile-time check | Create `Arc<dyn TxSource + Send + Sync>` from `InMemoryTxPool` | Code compiles, Arc usable across threads | P0 | [PROPOSED] |
| `InMemoryTxPool` existing tests pass | UT-APP-05 | Happy | Run existing test suite | Execute `cargo test -p app` | All tests in `tx_source.rs` (lines 54-132) pass unchanged | P0 | [GROUNDED: app/src/tx_source.rs:54-132] |

**Test location**: `crates/app/tests/tx_source_trait.rs` (new), `crates/app/src/tx_source.rs` (existing tests)

**Dependencies**: None (foundational crate)

---

### mempool

| Interface | Test case ID | Path (happy/failure) | Setup | Stimulus | Assertions/Oracle | Priority | Status |
|-----------|--------------|----------------------|-------|----------|-------------------|----------|--------|
| `MempoolStore::open()` creates new DB | UT-MEMPOOL-01 | Happy | Temp directory via `tempdir()` | `MempoolStore::open(path)` | DB opens successfully, `pending()` returns empty vec | P0 | [PROPOSED: mempool/tests/store_persistence.rs] |
| `MempoolStore::push()` persists single tx | UT-MEMPOOL-02 | Happy | Open store in temp dir | `push(vec![0x01, 0x02, 0x03])` | Subsequent `pending()` returns tx, DB file non-empty | P0 | [PROPOSED: mempool/tests/store_persistence.rs] |
| `MempoolStore::drain_pending()` empties DB | UT-MEMPOOL-03 | Happy | Push 3 txs to store | Call `drain_pending()` | Returns 3 txs in FIFO order, second `drain_pending()` returns empty vec | P0 | [PROPOSED: mempool/tests/store_persistence.rs] |
| `PersistentTxPool::open()` recovers after drop | UT-MEMPOOL-04 | Happy (crash recovery) | Open pool, push tx, drop pool | Open new pool instance with same path | New instance's `pending()` returns persisted tx | P0 | [PROPOSED: mempool/tests/store_persistence.rs] |
| `PersistentTxPool::push()` auto-increment keys | UT-MEMPOOL-05 | Happy | Open pool, push 5 txs with identical content | Call `pending()` | Returns 5 txs (duplicates preserved), FIFO order maintained | P1 | [PROPOSED: mempool/tests/persistent_trait.rs] |
| `PersistentTxPool::pending()` FIFO ordering | UT-MEMPOOL-06 | Happy | Open pool | Push txs `[0x01]`, `[0x02]`, `[0x03]` in sequence | `pending()` returns `[[0x01], [0x02], [0x03]]` (insertion order) | P0 | [PROPOSED: mempool/tests/persistent_trait.rs] |
| Concurrent `push()` thread-safety | UT-MEMPOOL-07 | Happy (stress test) | Create `Arc<PersistentTxPool>` | Spawn 10 threads, each pushes unique tx | All 10 txs present in `pending()`, no panics/data races | P0 | [PROPOSED: mempool/tests/persistent_trait.rs] |
| Concurrent push + drain | UT-MEMPOOL-08 | Happy (stress test) | Create `Arc<PersistentTxPool>` | Thread A pushes 100 txs, Thread B calls `drain_pending()` mid-push | No data corruption, Mutex prevents races, all committed txs either in drain or next drain | P1 | [PROPOSED: mempool/tests/concurrent.rs] |
| `MempoolStore::open()` fails on bad path | UT-MEMPOOL-09 | Failure | Path `/dev/null/mempool` (invalid) | `MempoolStore::open(path)` | Returns `Err(MempoolError::DatabaseOpen)`, diagnostic message | P1 | [PROPOSED: mempool/tests/error_cases.rs] |
| `push()` on read-only filesystem logs error | UT-MEMPOOL-10 | Failure (graceful degradation) | Open DB, remount read-only (mock) | Call `push(tx)` | No panic, error logged, subsequent `pending()` returns empty (tx dropped) | P2 | [PROPOSED: mempool/tests/error_cases.rs] |
| Empty DB `drain_pending()` returns empty vec | UT-MEMPOOL-11 | Happy (edge case) | Open fresh pool | Call `pending()` immediately | Returns empty vec, no errors | P1 | [PROPOSED: mempool/tests/store_persistence.rs] |
| Large tx (10KB) persists correctly | UT-MEMPOOL-12 | Happy (edge case) | Open pool | Push tx with 10,000 bytes | `pending()` returns full tx, no truncation | P2 | [PROPOSED: mempool/tests/edge_cases.rs] |
| Counter resumes after crash | UT-MEMPOOL-13 | Happy (crash recovery) | Push tx with key=5, drop, reopen | Push another tx | New tx has key=6 (counter resumed), FIFO preserved | P0 | [PROPOSED: mempool/tests/store_persistence.rs] |

**Test location**: `crates/mempool/tests/` (new test crate)

**Test modules**:
- `store_persistence.rs` — MDBX low-level operations
- `persistent_trait.rs` — TxSource trait compliance
- `concurrent.rs` — Thread-safety validation
- `error_cases.rs` — Error handling paths
- `edge_cases.rs` — Large payloads, boundary conditions

**Dependencies**: `tempdir` for test isolation, `parking_lot` for Mutex

---

### rpc-eth

| Interface | Test case ID | Path (happy/failure) | Setup | Stimulus | Assertions/Oracle | Priority | Status |
|-----------|--------------|----------------------|-------|----------|-------------------|----------|--------|
| `EthRpcContext` accepts trait object | UT-RPC-01 | Happy | Create `Arc<dyn TxSource>` from `InMemoryTxPool` | `EthRpcContext::new(tx_pool, ...)` | Context constructs successfully, field type is trait object | P0 | [PROPOSED: rpc-eth/tests/context_trait_object.rs] |
| `send_raw_transaction()` calls trait `push()` | UT-RPC-02 | Happy | Create context with mock trait object | Call `send_raw_transaction(bytes)` | Verify `push()` called on trait object (via mock recording) | P0 | [PROPOSED: rpc-eth/tests/context_trait_object.rs] |
| Existing RPC tests pass unchanged | UT-RPC-03 | Happy | Run existing test suite | Execute `cargo test -p rpc-eth` | All tests in `eth_handler.rs` (lines 206-558) pass with trait object field | P0 | [GROUNDED: rpc-eth/src/eth_handler.rs:206-558] |
| Test helper `test_ctx()` uses trait object | UT-RPC-04 | Happy | Update test helper (line 306-311) | Create context, verify type signature | `Arc<InMemoryTxPool>` casts to `Arc<dyn TxSource>` automatically | P1 | [PROPOSED: rpc-eth/src/eth_handler.rs:306-311] |

**Test location**: `crates/rpc-eth/tests/context_trait_object.rs` (new), `crates/rpc-eth/src/eth_handler.rs` (existing tests)

**Dependencies**: `app` for `InMemoryTxPool`, `state-memory` for mock state

---

### whirlpool-node

| Interface | Test case ID | Path (happy/failure) | Setup | Stimulus | Assertions/Oracle | Priority | Status |
|-----------|--------------|----------------------|-------|----------|-------------------|----------|--------|
| Node compiles with `PersistentTxPool` wiring | UT-NODE-01 | Happy (compile-time) | Add `mempool` dependency | `cargo build -p whirlpool-node` | Compilation succeeds, no type errors | P0 | [PROPOSED: build verification] |
| Mempool path computed correctly | UT-NODE-02 | Happy | Mock config with `DEFAULT_RUNTIME_STORAGE_DIR="/tmp/data"` | Compute `mempool_path = dir.join("mempool")` | Assert `mempool_path == "/tmp/data/mempool"` | P1 | [PROPOSED: whirlpool-node/tests/path_computation.rs] |
| `PersistentTxPool::open()` failure exits cleanly | UT-NODE-03 | Failure | Mock `open()` to return `Err(...)` | Node startup | Process exits with code 1, error logged with diagnostic | P1 | [PROPOSED: whirlpool-node/tests/startup_error.rs] |

**Test location**: `crates/whirlpool-node/tests/` (new tests)

**Dependencies**: `mempool`, `app`, integration test harness

---

## Integration Tests

### Cross-Crate Flow: Transaction Submission → Persistence

| Flow | Test case ID | Crates involved | Setup | Stimulus | Assertions/Oracle | Real vs Mocked deps | Priority | Status |
|------|--------------|-----------------|-------|----------|-------------------|---------------------|----------|--------|
| RPC → PersistentTxPool → storage | INT-FLOW-01 | `rpc-eth`, `mempool` | Start node with temp storage, create RPC client | Submit tx via `eth_sendRawTransaction` | TX hash returned, verify MDBX DB file contains tx (query via second pool instance) | Real: MDBX. Mocked: state, consensus | P0 | [PROPOSED: integration-tests/tests/mempool_rpc.rs] |
| RPC → InMemoryTxPool (regression) | INT-FLOW-02 | `rpc-eth`, `app` | Create context with `InMemoryTxPool` trait object | Submit tx via RPC handler | TX stored in memory pool, `pending()` returns it | Real: InMemoryTxPool. Mocked: state | P0 | [PROPOSED: rpc-eth/tests/integration.rs] |

---

### Cross-Crate Flow: Transaction Drain → Execution

| Flow | Test case ID | Crates involved | Setup | Stimulus | Assertions/Oracle | Real vs Mocked deps | Priority | Status |
|------|--------------|-----------------|-------|----------|-------------------|---------------------|----------|--------|
| Consensus → PersistentTxPool → EvmApplication | INT-FLOW-03 | `app-evm`, `mempool`, `consensus` | Push txs to persistent pool, create `EvmApplication` with pool | Call `app.propose(genesis, 1)` | Block contains drained txs, pool empty after propose | Real: MDBX, REVM. Mocked: consensus callbacks | P0 | [PROPOSED: app-evm/tests/integration_persistent.rs] |
| FIFO ordering preserved in proposal | INT-FLOW-04 | `app-evm`, `mempool` | Push 3 txs to pool: A, B, C | Call `propose()` | Block txs in order [A, B, C], matches insertion order | Real: MDBX, REVM. Mocked: state | P0 | [PROPOSED: app-evm/tests/integration_persistent.rs] |
| Drain with invalid EIP-2718 tx skips gracefully | INT-FLOW-05 | `app-evm`, `mempool` | Push 1 valid tx, 1 malformed tx (garbage bytes) | Call `propose()` | Block contains only valid tx, malformed tx skipped, error logged | Real: MDBX, REVM. Mocked: state | P1 | [PROPOSED: app-evm/tests/integration_persistent.rs] |

---

### Cross-Crate Flow: Node Startup/Recovery

| Flow | Test case ID | Crates involved | Setup | Stimulus | Assertions/Oracle | Real vs Mocked deps | Priority | Status |
|------|--------------|-----------------|-------|----------|-------------------|---------------------|----------|--------|
| Fresh node startup with empty DB | INT-WIRE-01 | `whirlpool-node`, `mempool` | Temp storage dir with no existing mempool DB | Start node via `main()` | Node starts successfully, mempool empty, RPC/consensus operational | Real: MDBX, state. Mocked: network | P0 | [PROPOSED: whirlpool-node/tests/integration_startup.rs] |
| Node startup with existing DB | INT-WIRE-02 | `whirlpool-node`, `mempool` | Pre-populate mempool DB with 5 txs, start node | Node startup | Mempool opens existing DB, next `propose()` drains 5 txs | Real: MDBX, state. Mocked: network | P0 | [PROPOSED: whirlpool-node/tests/integration_startup.rs] |

---

### Cross-Crate Flow: Crash Recovery

| Flow | Test case ID | Crates involved | Setup | Stimulus | Assertions/Oracle | Real vs Mocked deps | Priority | Status |
|------|--------------|-----------------|-------|----------|-------------------|---------------------|----------|--------|
| Crash after push, before drain | INT-CR-01 | `whirlpool-node`, `mempool`, `app-evm` | Start node, submit tx via RPC, drop node (simulated crash) | Restart node, trigger propose | Recovered tx included in first proposed block | Real: MDBX, state. Mocked: network | P0 | [PROPOSED: integration-tests/tests/mempool_persistence.rs] |
| Crash during drain (before commit) | INT-CR-02 | `mempool`, `app-evm` | Push tx, begin `drain_pending()` (mock crash before MDBX commit) | Reopen pool, call `pending()` | TX still in DB (uncommitted drain rolled back), recovered | Real: MDBX. Mocked: crash injection | P0 | [PROPOSED: mempool/tests/crash_recovery.rs] |
| Crash after drain, before finalization | INT-CR-03 | `whirlpool-node`, `mempool`, `consensus` | Push tx, drain via `propose()`, crash before finalization callback | Restart node | TX lost (drain committed, not finalized) — matches InMemoryTxPool behavior | Real: MDBX, consensus. Mocked: network | P0 | [PROPOSED: integration-tests/tests/mempool_persistence.rs] |
| Clean shutdown → restart | INT-CR-04 | `whirlpool-node`, `mempool` | Start node, submit tx, graceful shutdown (SIGTERM) | Restart node | TX recovered, MDBX clean close, no corruption | Real: MDBX, state. Mocked: network | P1 | [PROPOSED: integration-tests/tests/mempool_persistence.rs] |

**Test location**: `crates/integration-tests/tests/mempool_persistence.rs` (new integration test suite)

**Dependencies**: Full node stack (`whirlpool-node`, `mempool`, `app-evm`, `rpc-eth`, `state-reth`), `tempdir` for isolated storage

---

### End-to-End: Full Stack Persistence

| Flow | Test case ID | Entry -> Exit | Setup | Stimulus | Assertions/Oracle | Real vs Mocked deps | Priority | Status |
|------|--------------|---------------|-------|----------|-------------------|---------------------|----------|--------|
| RPC submit → crash → restart → propose → finalize | INT-E2E-01 | RPC → mempool → crash → recovery → consensus → state | Start full node with persistent storage | (1) Submit tx via RPC, (2) crash node, (3) restart node, (4) trigger consensus propose, (5) verify finalized block | TX included in finalized block, persisted in state DB, RPC receipt queryable | Real: all components except network. Mocked: network (single-node consensus) | P0 | [PROPOSED: integration-tests/tests/e2e_persistence.rs] |
| Multiple txs survive restart, FIFO preserved | INT-E2E-02 | RPC → mempool → crash → recovery → consensus | Submit 10 txs (A-J) via RPC, crash, restart, propose | All 10 txs in proposed block, order preserved: [A, B, ..., J] | Real: all except network. Mocked: network | P0 | [PROPOSED: integration-tests/tests/e2e_persistence.rs] |
| State mutation persists after crash | INT-E2E-03 | RPC → mempool → consensus → state → crash → recovery | Submit transfer tx (Alice → Bob 1 ETH), finalize, crash, restart | Query Bob's balance via RPC | Balance = 1 ETH (state persisted), mempool empty after finalization | Real: all except network. Mocked: network | P0 | [PROPOSED: integration-tests/tests/e2e_persistence.rs] |

**Test location**: `crates/integration-tests/tests/e2e_persistence.rs`

**Dependencies**: Full node, persistent state+mempool, RPC client

**Evidence**: Test patterns based on `app-evm/tests/integration.rs` (lines 28-119) full propose-verify cycle.

---

## Property-Based Invariants

| Invariant | Test case ID | Verification method | Setup | Stimulus | Assertions/Oracle | Priority | Status |
|-----------|--------------|---------------------|-------|----------|-------------------|----------|--------|
| FIFO ordering holds for N txs | PROP-01 | Property test (proptest crate) | Generate N random txs (1-100) | Push all, call `pending()` | Returned order matches insertion order for all N | P0 | [PROPOSED: mempool/tests/property.rs] |
| Concurrent push preserves all txs | PROP-02 | Property test | Generate M threads × N txs each | Concurrent push from all threads | Total drained txs = M × N, no duplicates/losses | P0 | [PROPOSED: mempool/tests/property.rs] |
| Counter monotonically increases | PROP-03 | Invariant check | Push-drain cycles in loop | Observe internal counter (requires test-only accessor) | Counter always increments, never decreases or reuses keys | P1 | [PROPOSED: mempool/tests/property.rs] |
| Drain is atomic (all-or-nothing) | PROP-04 | Crash injection during drain | Begin drain, inject mock crash at random point | Reopen pool | Either all txs drained (commit succeeded) or none (rollback) — no partial state | P0 | [PROPOSED: mempool/tests/crash_atomicity.rs] |

**Test location**: `crates/mempool/tests/property.rs`, `crates/mempool/tests/crash_atomicity.rs`

**Dependencies**: `proptest` crate for property-based testing

---

## Open Questions

### Test Environment Setup
1. **MDBX path isolation**: Each test must use unique temp directory to avoid cross-test interference. Use `tempdir::TempDir` per test case.
2. **Async runtime**: Integration tests with RPC require `#[tokio::test]`. Mempool unit tests can use sync `#[test]`.
3. **Crash simulation**: Use `std::mem::drop()` to simulate clean drop, `panic!()` for unclean crash (not in test harness — separate binary).

### Coverage Targets
- **Unit test coverage**: Aim for 100% of public API surface (`PersistentTxPool::open`, `push`, `pending`)
- **Integration test coverage**: All flows in FLOWS.md (5 flows) covered by at least one test
- **Crash recovery**: ACID guarantees validated for critical scenarios (INT-CR-01, INT-CR-02, PROP-04)

### Test Data Management
- **Temp directories**: Always use `tempdir()`, never hardcode paths like `/tmp/test-mempool`
- **Cleanup**: TempDir auto-deletes on drop. For manual cleanup, use `defer!` or RAII patterns.
- **Seed data**: Use deterministic test transactions (fixed nonces, addresses) for reproducibility.

### Future Enhancements (Out of Scope for MVP)
- **Performance benchmarks**: Measure push/drain latency vs InMemoryTxPool (expect <1ms overhead)
- **Fuzzing**: Use `cargo fuzz` to test MDBX operations with random inputs
- **Observability tests**: Verify tracing spans/events emitted correctly (requires test subscriber)
- **Lifecycle tracking tests**: When proposed_txs table added, test re-queue on crash (future enhancement)

---

## Test Dependency Table

| Test Crate/Module | Dependencies | External Tools |
|-------------------|--------------|----------------|
| `app/tests/` | `app` (self) | None |
| `mempool/tests/` | `app` (trait), `libmdbx-rs`, `tempdir`, `parking_lot` | MDBX runtime |
| `rpc-eth/tests/` | `app`, `state-memory`, `jsonrpsee` | None (mocked network) |
| `app-evm/tests/` | `app`, `app-evm`, `mempool`, `state-memory`, `reth-primitives` | REVM, MDBX |
| `integration-tests/tests/` | Full workspace (all crates) | MDBX, tokio runtime |
| `whirlpool-node/tests/` | `whirlpool-node`, `mempool`, `app`, `state-reth` | MDBX, tokio runtime |

**Shared Test Utilities** (proposed):
- `test-utils` crate with:
  - `fn temp_mempool_path() -> TempDir` — isolation helper
  - `fn sample_tx(nonce: u64) -> Vec<u8>` — deterministic test tx generator
  - `fn mock_tx_source(txs: Vec<Vec<u8>>) -> Arc<dyn TxSource>` — trait object helper

---

## Test Execution Order (Recommended)

1. **Phase 1: Unit tests (app)** — Verify trait extension compiles
   - `cargo test -p app`
   - Gate: All existing + new trait tests pass

2. **Phase 2: Unit tests (mempool)** — Verify MDBX operations + trait impl
   - `cargo test -p mempool`
   - Gate: Store persistence + crash recovery tests pass

3. **Phase 3: Unit tests (rpc-eth)** — Verify trait object generification
   - `cargo test -p rpc-eth`
   - Gate: All existing tests pass with trait object field

4. **Phase 4: Integration tests (flows)** — Verify cross-crate interactions
   - `cargo test -p app-evm --test integration_persistent`
   - `cargo test -p whirlpool-node --test integration_startup`
   - Gate: RPC → mempool → consensus flows pass

5. **Phase 5: End-to-end tests** — Full stack validation
   - `cargo test -p integration-tests --test e2e_persistence`
   - Gate: Crash recovery + state persistence verified

6. **Phase 6: Property tests** — Stress testing + invariant validation
   - `cargo test -p mempool --test property`
   - Gate: FIFO + concurrent safety invariants hold

7. **Full workspace test** — Final validation
   - `cargo test` (all crates)
   - Gate: Zero regressions, all new tests pass

---

## Acceptance Criteria Summary

### Per-Crate Gates
- **app**: Trait extension compiles, `InMemoryTxPool`/`NoopTxSource` updated, existing tests pass
- **mempool**: Push/drain/recovery tests pass, concurrent access safe, FIFO ordering verified
- **rpc-eth**: Trait object field works, all RPC methods unchanged, test helpers updated
- **whirlpool-node**: Compilation succeeds, mempool path correct, startup error handling works

### Integration Gates
- **RPC → mempool**: Submitted tx persists to MDBX
- **Mempool → consensus**: Drained txs execute in REVM
- **Crash recovery**: Txs survive restart (committed before crash)
- **End-to-end**: RPC submission → crash → restart → finalization → state persistence

### Critical Path Tests (Must Pass)
1. UT-MEMPOOL-04 (crash recovery)
2. INT-CR-01 (crash after push, before drain)
3. INT-E2E-01 (full stack with crash)
4. PROP-04 (drain atomicity)

All other tests are supporting validation for these core scenarios.
