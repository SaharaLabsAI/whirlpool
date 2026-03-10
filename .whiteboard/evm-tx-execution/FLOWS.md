# Flows — EVM Transaction Execution

## F1: Block Proposal with EVM Execution

**Trigger**: Consensus engine calls `ApplicationAdapter::propose(parent, height)` → `EvmApplication::propose(parent, height)`

**Preconditions**: Parent `EvmBlock` available, `InMemoryStateDb` reflects state after parent block.

### Happy Path

```
1. tx_source.pending() → Vec<Vec<u8>>          [Grounded: crates/app/src/traits.rs::TxSource]
2. decode_transactions(raw_txs)                  [PROPOSED]
   → TransactionSigned::decode_2718 per tx
   → try_recover sender per tx
   → collect Vec<RecoveredTx>, skip decode failures
3. snapshot = state_db.read().clone()             [PROPOSED: clone for rollback safety]
4. state = reth_revm::State::builder()
     .with_database(snapshot)
     .with_bundle_update()
     .build()                                     [PROPOSED]
5. parent_header = build_sealed_header(parent)    [Grounded: crates/app-evm/src/executor.rs]
6. attrs = NextBlockEnvAttributes {               [PROPOSED]
     timestamp: parent.timestamp + 12,
     suggested_fee_recipient: Address::ZERO,
     prev_randao: B256::ZERO,
     gas_limit: 30_000_000,
     parent_beacon_block_root: None,
     withdrawals: vec![],
     extra_data: Bytes::new(),
   }
7. builder = evm_config.builder_for_next_block(&mut state, &parent_header, attrs)
8. builder.apply_pre_execution_changes()
9. for tx in recovered_txs:
     match builder.execute_transaction(tx):
       Ok(gas) → accumulate gas_used
       Err(_) → skip tx, log warning              [PROPOSED: D-4 skip invalid]
10. bundle_state = state.take_bundle()
11. canonical_db.write().commit(&bundle_state)     [Grounded: crates/state/src/db.rs::commit]
12. state_root = canonical_db.read().state_root()  [Grounded: crates/state/src/db.rs::state_root]
13. tx_root = compute_tx_root(executed_txs)        [PROPOSED: alloy-trie]
14. receipts_root = compute_receipts_root(receipts) [PROPOSED: alloy-trie]
15. block = EvmBlock { height, parent_id: parent.id(),
      state_root, tx_root, receipts_root, gas_used,
      timestamp: attrs.timestamp,
      transactions: executed_tx_bytes }            [Grounded: crates/app/src/types.rs::EvmBlock]
16. return Ok(block)
```

### Error Paths

| Error | Source | Handling |
|---|---|---|
| All txs fail to decode | Step 2 | Return empty block (valid — no transactions) |
| EVM execution error (non-tx) | Steps 7-8 | Return `Err(ApplicationError::Execution)` |
| State commit failure | Step 11 | Return `Err(ApplicationError::State)` (currently infallible) |
| State root computation failure | Step 12 | Return `Err(ApplicationError::State)` |

### Rollback on Consensus Rejection

If consensus rejects the proposed block, canonical state (step 11) has already been mutated. **[PROPOSED]** mitigation: step 3 clones state; step 11 commits to canonical only after successful execution. If proposal fails mid-execution, the clone is discarded and canonical state is untouched.

**Open issue**: If proposal succeeds but consensus later rejects, canonical state IS committed. Full rollback requires a finalization callback (not available in current ConsensusApp trait — `crates/consensus/src/app.rs::ConsensusApp`). For MVP, this is accepted as a limitation in single-proposer mode.

---

## F2: Block Verification with EVM Re-execution

**Trigger**: Consensus engine calls `ApplicationAdapter::verify(parent, block)` → `EvmApplication::verify(parent, block)`

**Preconditions**: Parent `EvmBlock` and candidate `block` available. `InMemoryStateDb` reflects state after parent.

### Happy Path

```
1. decode_transactions(block.transactions)         [PROPOSED: same helper as F1]
   → Vec<RecoveredTx> (ALL must decode; fail = invalid block)
2. snapshot = state_db.read().clone()               [PROPOSED: isolation from canonical]
3. state = reth_revm::State::builder()
     .with_database(snapshot)
     .with_bundle_update()
     .build()                                       [PROPOSED]
4. Reconstruct RecoveredBlock from block fields      [PROPOSED]
5. executor = BasicBlockExecutor::new(evm_config, state)
6. result = executor.execute_one(&recovered_block)   [PROPOSED]
   → BlockExecutionResult { receipts, gas_used, requests }
7. bundle_state = executor.into_state().take_bundle()
8. Apply bundle to snapshot clone → compute state_root [PROPOSED]
9. Compute tx_root, receipts_root from results        [PROPOSED]
10. Compare:
    - computed.state_root == block.state_root
    - computed.tx_root == block.tx_root
    - computed.receipts_root == block.receipts_root
    - computed.gas_used == block.gas_used
11. All match → return Ok(())
12. Mismatch → return Err(EvmAppError::StateRootMismatch or InvalidBlock)
```

### Error Paths

| Error | Source | Handling |
|---|---|---|
| Transaction decode failure | Step 1 | `Err(InvalidBlock)` — all txs must be decodable |
| EVM execution failure | Step 6 | `Err(Execution)` |
| Field mismatch | Step 10 | `Err(StateRootMismatch)` or `Err(InvalidBlock)` with details |

### Key Difference from F1

- Verify does NOT commit to canonical state (operates entirely on clone)
- Verify requires ALL transactions to decode (propose can skip invalid ones)
- Verify uses batch executor, not incremental builder

---

## F3: State Commitment

**Trigger**: Successful EVM execution produces a `BundleState`.

### Flow

```
1. InMemoryStateDb::commit(&bundle_state)
   a. For each (addr, account) in bundle.state:
      - Destroyed → remove account + clear storage
      - Created/Changed → update nonce, balance, code_hash; apply storage diffs
   b. For each (hash, bytecode) in bundle.contracts:
      - Insert into bytecodes map
2. InMemoryStateDb::insert_block_hash(height, block.id())
3. InMemoryStateDb::state_root() → B256
```

All steps grounded: `crates/state/src/db.rs::InMemoryStateDb::commit`, `::insert_block_hash`, `::state_root`.

---

## Implementation Slices

| Slice | Flows | Dependencies |
|---|---|---|
| S1: Transaction decode/recover helper | F1.2, F2.1 | None |
| S2: Propose execution flow | F1.3-F1.15, F3 | S1 |
| S3: State snapshot (Clone) | F1.3, F2.2 | None |
| S4: Verify re-execution flow | F2.2-F2.12 | S1, S3 |
| S5: Block field computation (tx_root, receipts_root) | F1.13-F1.14, F2.9 | S2 |
| S6: Integration tests | F1+F2 round-trip | S1-S5 |
