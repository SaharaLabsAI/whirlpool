# Proven Acceptance Criteria

**AC_VERSION:** 1

**Feature:** Persistent State with reth-db

**Design Docs Root:** `/home/dev/sahara/web3/agent/playground/whirlpool/.design-scratch/e2e/persistent-state-rethdb-20260305-1347/docs/`

**Proof Document:** `/home/dev/sahara/web3/agent/playground/whirlpool/.design-scratch/e2e/persistent-state-rethdb-20260305-1347/main/proof.md`

---

## Acceptance Criteria (AC)

All acceptance criteria are P0 (critical) and MUST pass before feature merge.

| ID | Description | Verification | Expected Result |
|---|---|---|---|
| AC-1 | state-reth crate compiles | `nix develop --command cargo build -p state-reth` | Exit code 0, no compilation errors |
| AC-2 | state-reth unit tests pass | `nix develop --command cargo test -p state-reth` | All tests pass (0 failures) |
| AC-3 | state crate compiles (trait migration) | `nix develop --command cargo test -p state` | All tests pass; trait signature is fallible |
| AC-4 | state-memory adapts to fallible trait | `nix develop --command cargo test -p state-memory` | All tests pass; `type Error = Infallible` |
| AC-5 | app-evm consumer compiles | `nix develop --command cargo test -p app-evm` | All tests pass; no compilation errors |
| AC-6 | rpc-eth consumer compiles | `nix develop --command cargo test -p rpc-eth` | All tests pass; no compilation errors |
| AC-7 | whirlpool-node wiring compiles | `nix develop --command cargo test -p whirlpool-node` | All tests pass; wiring is correct |
| AC-8 | State persists across restarts | Integration test TC-SR-I001 (test_commit_durability) | Write state → close DB → reopen → read state → data matches |
| AC-9 | Genesis bootstrap correct | Integration tests TC-SR-I005, TC-SR-I006, TC-WN-I002 | Genesis populates accounts/storage/code; state root computed |
| AC-10 | BundleState commit works | Unit tests TC-SR-U003, TC-SR-U005, TC-SR-U007 + integration TC-SR-I001 | Commit changes → read back → all changes visible |
| AC-11 | Full workspace build succeeds | `nix develop --command cargo build` | Exit code 0, all crates compile |
| AC-12 | Full workspace tests pass | `nix develop --command cargo test` | All tests pass (0 failures) |

---

## Quality Assurance Scenarios (QA)

| ID | Description | Verification | Priority | Expected Result |
|---|---|---|---|---|
| QA-1 | Concurrent reads during write safe | Integration test TC-SR-I004 (test_single_writer_multiple_readers) | P0 | Writer completes, readers see updated state, no panics/deadlocks |
| QA-2 | Large state (10K+ accounts) persists | Manual stress test or property test | P1 | All sampled accounts match expected values; no corruption |
| QA-3 | Empty BundleState commit is no-op | Unit test | P2 | Returns `Ok(())`; no error, no side effects |

---

## Invariants (INV)

All invariants are P0 (critical) and MUST hold at all times.

| ID | Statement | Verification | Domain |
|---|---|---|---|
| INV-1 | StateDb trait is fallible | Compile + test TC-ST-U001 | State Interface (D2) |
| INV-2 | InMemoryStateDb uses Infallible error | Static check + test TC-ST-U002 | State Interface (D2) |
| INV-3 | State persists across restarts | Test TC-SR-I001 + manual verification | Persistent Storage (D1) |
| INV-4 | state_root() is deterministic | Tests TC-SR-I007, TC-SR-I008, TC-CC-I004 | State Root (D3) |
| INV-5 | commit() is atomic | Tests TC-SR-I002, TC-CC-I006 | Persistent Storage (D1) + State Interface (D2) |
| INV-6 | RethStateDb is Clone+Send+Sync+Debug | Compile + tests TC-SR-I003, TC-SR-I004 | Persistent Storage (D1) + Node Wiring (D4) |
| INV-7 | Consumers compile with new trait | Compile checks (AC-5, AC-6) + unit tests | State Interface (D2) + Node Wiring (D4) |
| INV-8 | Genesis bootstrap is correct | Tests TC-SR-I005, TC-WN-I002, TC-CC-I003 | Node Wiring (D4) + Persistent Storage (D1) |

---

## Test Case References

### Unit Tests (state-reth)

- **TC-SR-U001:** test_create_db_success
- **TC-SR-U002:** test_init_db_tables_exist
- **TC-SR-U003:** test_insert_and_get_account
- **TC-SR-U004:** test_get_account_missing
- **TC-SR-U005:** test_insert_and_get_storage
- **TC-SR-U006:** test_get_storage_missing
- **TC-SR-U007:** test_insert_and_get_code
- **TC-SR-U008:** test_insert_and_get_block_hash
- **TC-SR-U009:** test_state_root_empty_state
- **TC-SR-U010:** test_state_root_with_accounts
- **TC-SR-U011:** test_revm_database_basic
- **TC-SR-U012:** test_revm_database_storage
- **TC-SR-U013:** test_revm_database_ref_basic
- **TC-SR-U014:** test_error_invalid_path
- **TC-SR-U015:** test_error_read_after_close
- **TC-SR-U016:** test_codec_round_trip_account
- **TC-SR-U017:** test_codec_round_trip_bytecode

### Unit Tests (state trait migration)

- **TC-ST-U001:** test_statedb_trait_fallible_signature
- **TC-ST-U002:** test_state_memory_infallible_impl

### Integration Tests (state-reth)

- **TC-SR-I001:** test_commit_durability
- **TC-SR-I002:** test_commit_rollback_on_error
- **TC-SR-I003:** test_concurrent_reads
- **TC-SR-I004:** test_single_writer_multiple_readers
- **TC-SR-I005:** test_with_genesis_populates_accounts
- **TC-SR-I006:** test_with_genesis_computes_root
- **TC-SR-I007:** test_state_root_determinism
- **TC-SR-I008:** test_state_root_idempotency

### Integration Tests (whirlpool-node)

- **TC-WN-I001:** test_node_startup_with_rethstatedb
- **TC-WN-I002:** test_genesis_initialization_on_first_startup
- **TC-WN-I003:** test_node_startup_fails_on_invalid_path
- **TC-WN-I004:** test_graceful_shutdown

### End-to-End Tests

- **TC-CC-I001:** EVM execution -> state persistence
- **TC-CC-I002:** RPC query over persistent state
- **TC-CC-I003:** Genesis -> Commit -> Read
- **TC-CC-I004:** State root consistency
- **TC-CC-I005:** MDBX error -> EVM execution abort
- **TC-CC-I006:** MDBX write failure -> commit rollback

---

## Verification Protocol

### Phase 1: Build Verification

1. Run `nix develop --command cargo build -p state-reth` (AC-1)
2. Run `nix develop --command cargo test -p state-reth` (AC-2)
3. Run `nix develop --command cargo test -p state` (AC-3)
4. Run `nix develop --command cargo test -p state-memory` (AC-4)
5. Run `nix develop --command cargo test -p app-evm` (AC-5)
6. Run `nix develop --command cargo test -p rpc-eth` (AC-6)
7. Run `nix develop --command cargo test -p whirlpool-node` (AC-7)
8. Run `nix develop --command cargo build` (AC-11)
9. Run `nix develop --command cargo test` (AC-12)

**Gate:** ALL build commands MUST succeed (exit code 0) before proceeding.

---

### Phase 2: Integration Testing

1. Verify TC-SR-I001 passes (AC-8)
2. Verify TC-SR-I005, TC-SR-I006, TC-WN-I002 pass (AC-9)
3. Verify TC-SR-U003, TC-SR-U005, TC-SR-U007, TC-SR-I001 pass (AC-10)
4. Verify TC-SR-I004 passes (QA-1)

**Gate:** ALL P0 integration tests MUST pass before feature merge.

---

### Phase 3: Invariant Validation

1. Verify INV-1 through INV-8 hold via test execution
2. Manual verification of INV-3 (restart persistence)
3. Manual verification of INV-4 (determinism across process restarts)

**Gate:** ALL invariants MUST be verified before feature acceptance.

---

## Feature Gate: Merge Criteria

**The feature is ready for merge when:**

1. ✅ ALL 12 acceptance criteria (AC-1 through AC-12) pass
2. ✅ ALL P0 QA scenarios pass (QA-1)
3. ✅ ALL 8 invariants are verified
4. ✅ Risk R-1 (MDBX prerequisites) is resolved or documented
5. ✅ Full workspace `cargo build` and `cargo test` succeed

**Total Verification Surface:**
- 12 acceptance criteria
- 3 QA scenarios (1 P0, 1 P1, 1 P2)
- 8 invariants
- 46 test cases (26 P0, 12 P1, 8 P2)

**Minimum Pass Threshold:** ALL P0 criteria + ALL P0 tests = 12 AC + 1 QA + 8 INV + 26 tests = **47 P0 verification points**
