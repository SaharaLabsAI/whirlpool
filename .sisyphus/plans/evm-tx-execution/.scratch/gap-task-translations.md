# Gap → Task Translations — EVM Transaction Execution

> Translates design slices (S-1..S-6) and test contracts (T-1..T-13) into concrete
> implementation tasks with TDD ordering. Each task is atomic and verifiable.

---

## Plan Mode: `full`
## Cross-check: N/A (no existing plan)

---

## Task Translation Table

### Wave 1: Foundation (No Dependencies)

#### Task 1: Add Clone derives to state types
- **Slice**: S-3
- **Crate**: `state`
- **File**: `crates/state/src/db.rs`
- **Change**: Add `#[derive(Clone)]` to `InMemoryStateDb` and `DbAccount`
- **TDD**: Write T-12 (clone_provides_independent_snapshot) first, then implement Clone
- **Tests**: T-12
- **Verify**: `cargo test -p state`
- **Effort**: XS
- **Blocks**: Task 4, Task 5

#### Task 2: State commit correctness tests
- **Slice**: S-3
- **Crate**: `state`
- **File**: `crates/state/src/db.rs` (test module)
- **Change**: Write T-9, T-10, T-11 to validate existing commit() behavior
- **TDD**: Tests only — commit() already exists, tests verify its correctness
- **Tests**: T-9 (account changes), T-10 (storage changes), T-11 (account destruction)
- **Verify**: `cargo test -p state`
- **Effort**: S
- **Blocks**: Task 4, Task 5

#### Task 3: Transaction decode/recover helper
- **Slice**: S-1
- **Crate**: `app-evm`
- **File**: `crates/app-evm/src/executor.rs`
- **Change**: Add `decode_transactions(&[Vec<u8>]) -> Result<Vec<RecoveredTx>, EvmAppError>`. Uses `TransactionSigned::decode_2718()` + `try_recover()`. Also add new deps to Cargo.toml if needed.
- **TDD**: Write unit test for decode (valid tx, invalid bytes, empty input) first, then implement
- **Tests**: Inline unit tests for decode_transactions
- **Verify**: `cargo test -p app-evm`
- **Effort**: S
- **Blocks**: Task 4, Task 5

### Wave 2: Core Execution (Depends on Wave 1)

#### Task 4: Propose execution flow + dependency wiring
- **Slice**: S-2, S-5
- **Crate**: `app-evm`
- **Files**: `crates/app-evm/src/executor.rs`, `crates/app-evm/Cargo.toml`
- **Change**: Replace propose() stub with full 11-step EVM execution flow:
  1. Fetch pending txs from tx_source
  2. decode_transactions (skip failures for propose)
  3. Clone state_db for snapshot
  4. Build reth_revm::State wrapper
  5. build_sealed_header from parent
  6. Create NextBlockEnvAttributes (defaults)
  7. builder_for_next_block
  8. apply_pre_execution_changes
  9. Execute txs (skip failures)
  10. take_bundle → commit to canonical
  11. Compute state_root, tx_root, receipts_root, gas_used → assemble EvmBlock
- **TDD**: Write T-7 (empty tx source) first → implement → T-1 (transfer) → T-2 (contract deploy) → T-3 (skip invalid)
- **Tests**: T-1, T-2, T-3, T-7
- **Verify**: `cargo test -p app-evm`, `cargo build -p app-evm`
- **Effort**: L
- **Blocks**: Task 5
- **Dependencies**: Task 1 (Clone), Task 3 (decode_transactions)
- **Blocker workarounds**: B-1 (bypass builder.finish, use take_bundle + state_root directly)

#### Task 5: Verify re-execution flow
- **Slice**: S-4
- **Crate**: `app-evm`
- **File**: `crates/app-evm/src/executor.rs`
- **Change**: Replace verify() stub with batch re-execution + 4-field comparison:
  1. decode_transactions (ALL must decode, fail = InvalidBlock)
  2. Clone state_db (isolation, never commit to canonical)
  3. Build reth_revm::State wrapper
  4. Reconstruct RecoveredBlock → BasicBlockExecutor::execute_one
  5. Compute state_root, tx_root, receipts_root, gas_used from execution result
  6. Compare all 4 fields against proposed block
- **TDD**: Write T-4 (accepts valid) first → implement → T-5 (wrong state root) → T-6 (undecodable txs) → T-8 (wrong gas)
- **Tests**: T-4, T-5, T-6, T-8
- **Verify**: `cargo test -p app-evm`, `cargo build -p app-evm`
- **Effort**: L
- **Dependencies**: Task 1 (Clone), Task 3 (decode_transactions), Task 4 (propose, for generating test blocks)

### Wave 3: Integration (Depends on Wave 2)

#### Task 6: Integration test — propose/verify round trip
- **Slice**: S-6
- **Crate**: `app-evm`
- **File**: `crates/app-evm/tests/integration.rs` or inline test module
- **Change**: Write T-13 (propose_verify_round_trip) — full genesis → propose → verify cycle with real TxSource, real InMemoryStateDb, real EVM
- **TDD**: Test only (all implementation done in prior tasks)
- **Tests**: T-13
- **Verify**: `cargo test -p app-evm`, full workspace `cargo build`
- **Effort**: M
- **Dependencies**: Task 4, Task 5

#### Task 7: Final verification — full workspace build + test
- **Slice**: N/A
- **Crate**: all
- **Change**: None — verification only
- **Action**: Run `cargo build --workspace` and `cargo test --workspace` to ensure no regressions
- **Verify**: Exit code 0 for both
- **Effort**: XS
- **Dependencies**: Task 6

#### Task 8: Update llmdocs for modified crates
- **Slice**: N/A
- **Crate**: app-evm, state
- **Change**: Run `ctx-update-doc` skill to generate/update llmdocs
- **Action**: Update llmdocs for app-evm and state crates after code changes
- **Verify**: llmdocs updated and consistent with code
- **Effort**: S
- **Dependencies**: Task 7

---

## Dependency Graph

```
Task 1 (Clone) ─────────────┐
                              ├──→ Task 4 (Propose) ──→ Task 5 (Verify) ──→ Task 6 (Integration) ──→ Task 7 (Full verify) ──→ Task 8 (llmdocs)
Task 2 (State tests) ────────┘                    ↗
                                                  /
Task 3 (Decode helper) ─────────────────────────/
```

## Parallelism Opportunities

- **Wave 1**: Tasks 1, 2, 3 can all run in parallel (no interdependencies)
- **Wave 2**: Tasks 4 and 5 are sequential (5 depends on 4 for test block generation)
- **Wave 3**: Tasks 6, 7, 8 are sequential

## TDD Ordering Summary

Each task follows strict Red-Green-Refactor:
1. Write failing test(s) first
2. Implement minimum code to pass
3. Refactor if needed
4. Verify with `cargo test`

## Effort Estimates

| Task | Effort | Lines Changed (est.) |
|------|--------|---------------------|
| 1 | XS | ~5 |
| 2 | S | ~80 |
| 3 | S | ~40 |
| 4 | L | ~200 |
| 5 | L | ~150 |
| 6 | M | ~60 |
| 7 | XS | 0 |
| 8 | S | N/A (doc gen) |
| **Total** | **XL** | **~535** |
