# Test Contracts — EVM Transaction Execution

## Success Criteria Mapping

| SC | Description | Tests |
|---|---|---|
| SC-1 | propose() executes txs and returns correct block | T-1, T-2, T-3, T-7 |
| SC-2 | verify() re-executes and validates block fields | T-4, T-5, T-8 |
| SC-3 | commit() processes BundleState correctly | T-9, T-10, T-11 |
| SC-4 | Invalid txs handled gracefully | T-3, T-6 |
| SC-5 | State not corrupted on rejected proposal | T-12 |
| SC-6 | Existing tests pass + new coverage | T-1 through T-13 |

## Unit Tests — app-evm

### T-1: propose_executes_transfer_transaction
**Crate**: app-evm | **Type**: Unit
**Setup**: InMemoryStateDb with genesis account (funded). TxSource returns one signed transfer tx.
**Action**: Call `propose(genesis_block, 1)`.
**Assert**: Returned EvmBlock has: gas_used > 0, tx_root ≠ EMPTY_ROOT_HASH, receipts_root ≠ EMPTY_ROOT_HASH, state_root reflects balance changes, transactions contains the executed tx bytes.

### T-2: propose_executes_contract_deployment
**Crate**: app-evm | **Type**: Unit
**Setup**: InMemoryStateDb with funded deployer account. TxSource returns one contract creation tx.
**Action**: Call `propose(genesis_block, 1)`.
**Assert**: EvmBlock has gas_used > 0, state_root reflects new contract account, receipts show contract address.

### T-3: propose_skips_invalid_transactions
**Crate**: app-evm | **Type**: Unit
**Setup**: TxSource returns [valid_tx, invalid_bytes, valid_tx2].
**Action**: Call `propose(genesis_block, 1)`.
**Assert**: EvmBlock contains only the two valid txs. gas_used reflects only valid executions. No error returned.

### T-4: verify_accepts_valid_block
**Crate**: app-evm | **Type**: Unit
**Setup**: Produce block via `propose()`.
**Action**: Call `verify(genesis_block, proposed_block)`.
**Assert**: Returns `Ok(())`.

### T-5: verify_rejects_wrong_state_root
**Crate**: app-evm | **Type**: Unit
**Setup**: Produce block via `propose()`, tamper with `state_root` field.
**Action**: Call `verify(genesis_block, tampered_block)`.
**Assert**: Returns `Err(StateRootMismatch)`.

### T-6: verify_rejects_undecodable_transactions
**Crate**: app-evm | **Type**: Unit
**Setup**: Construct EvmBlock with invalid bytes in transactions field.
**Action**: Call `verify(genesis_block, bad_block)`.
**Assert**: Returns `Err(InvalidBlock)`.

### T-7: propose_empty_txsource_produces_empty_block
**Crate**: app-evm | **Type**: Unit
**Setup**: NoopTxSource (returns empty vec).
**Action**: Call `propose(genesis_block, 1)`.
**Assert**: EvmBlock has gas_used = 0, tx_root = EMPTY_ROOT_HASH, receipts_root = EMPTY_ROOT_HASH, state_root unchanged from parent.

### T-8: verify_rejects_wrong_gas_used
**Crate**: app-evm | **Type**: Unit
**Setup**: Produce block, tamper with gas_used.
**Action**: Call `verify(genesis_block, tampered_block)`.
**Assert**: Returns `Err(InvalidBlock)`.

## Unit Tests — state

### T-9: commit_applies_account_changes
**Crate**: state | **Type**: Unit
**Setup**: InMemoryStateDb with one account.
**Action**: Commit BundleState with balance increase.
**Assert**: Database read shows updated balance. state_root changed.

### T-10: commit_applies_storage_changes
**Crate**: state | **Type**: Unit
**Setup**: InMemoryStateDb with contract account.
**Action**: Commit BundleState with storage slot write.
**Assert**: Database read shows new storage value.

### T-11: commit_handles_account_destruction
**Crate**: state | **Type**: Unit
**Setup**: InMemoryStateDb with account to destroy.
**Action**: Commit BundleState with destroy flag.
**Assert**: Account no longer readable via Database trait. state_root changed.

### T-12: clone_provides_independent_snapshot
**Crate**: state | **Type**: Unit
**Setup**: InMemoryStateDb with accounts.
**Action**: Clone, commit to clone, verify original unchanged.
**Assert**: Original state_root unchanged. Clone state_root changed.

## Integration Tests

### T-13: propose_verify_round_trip
**Crate**: app-evm | **Type**: Integration
**Setup**: Full EvmApplication with InMemoryStateDb (genesis-funded accounts) and a TxSource returning multiple signed txs.
**Action**: `genesis()` → `propose(genesis, 1)` → `verify(genesis, proposed_block)`.
**Assert**: verify returns Ok. Block fields are internally consistent. State reflects all executed transactions.

## Cross-Crate Test Seams

| Seam | Real | Mocked |
|---|---|---|
| TxSource | T-13 (custom impl) | T-1 through T-8 (mock returning fixed txs) |
| InMemoryStateDb | All tests (real) | None (always real DB) |
| reth EVM | All tests (real execution) | None (always real EVM) |
| Consensus | Not tested here | Out of scope (Sub-Intent 3) |
