# Shared Wiring Skeleton — EVM Transaction Execution

## Block Proposal Flow (propose)

```
consensus engine
  → ApplicationAdapter::propose(parent, height)
    → EvmApplication::propose(parent, height)
      1. tx_source.pending() → Vec<Vec<u8>> (raw txs)
      2. decode & recover senders: TransactionSigned::decode_2718 + try_recover
      3. [PROPOSED] build reth State<DB> wrapper around InMemoryStateDb
      4. [PROPOSED] builder = evm_config.builder_for_next_block(&mut state, &parent_header, attrs)
      5. [PROPOSED] builder.apply_pre_execution_changes()
      6. [PROPOSED] for tx in recovered_txs: builder.execute_transaction(tx)
      7. [PROPOSED] outcome = builder.finish(state_provider)
         → BlockBuilderOutcome { execution_result, hashed_state, trie_updates, block }
      8. [PROPOSED] extract BundleState from state, commit to InMemoryStateDb
      9. [PROPOSED] compute state_root after commit
      10. assemble EvmBlock { height, parent_id, state_root, tx_root, receipts_root, gas_used, timestamp, transactions }
    → return EvmBlock
  → consensus finalizes
```

## Block Verification Flow (verify)

```
consensus engine
  → ApplicationAdapter::verify(parent, block)
    → EvmApplication::verify(parent, block)
      1. decode & recover txs from block.transactions
      2. [PROPOSED] build reth State<DB> wrapper (snapshot of current state? or re-read?)
      3. [PROPOSED] re-execute all transactions deterministically
         - option A: use BasicBlockExecutor::execute_one with reconstructed RecoveredBlock
         - option B: use builder pattern same as propose
      4. [PROPOSED] compare computed state_root, tx_root, receipts_root, gas_used vs block fields
      5. if mismatch → EvmAppError::StateRootMismatch or InvalidBlock
      6. [DECISION NEEDED] do we commit state on verify? or only on propose?
    → return Ok(()) or Err
```

## State Commit Flow

```
[After successful propose or verify]
  1. InMemoryStateDb::commit(&BundleState)
     - process account changes (create/update/destroy)
     - process storage changes
     - insert new bytecodes
  2. InMemoryStateDb::insert_block_hash(height, block_id)
  3. InMemoryStateDb::state_root() → B256 (flat keccak256)
```

## Open Questions

- **Q1**: Propose mutates state (commit). If propose succeeds but consensus rejects the block, state is corrupted. Need snapshot/rollback. → BLOCKER (decision-gap)
- **Q2**: Verify path — should it re-execute on a temporary snapshot, or does it also commit? Current codebase has no snapshot mechanism. → BLOCKER (decision-gap)
- **Q3**: The reth `builder.finish()` expects a `state_provider` for `state_root_with_updates`. Our `InMemoryStateDb` doesn't impl this reth trait. We'd need to compute state_root ourselves post-commit instead. → BLOCKER (information-gap)
- **Q4**: `NextBlockEnvAttributes` requires `suggested_fee_recipient`, `prev_randao`, `parent_beacon_block_root`, `withdrawals`. What values for Sahara chain? → information-gap
