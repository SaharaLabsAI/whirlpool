# Strategy — EVM Transaction Execution

## Architecture Direction

Replace the empty-block stubs in `app-evm::EvmApplication` with real EVM transaction execution, using reth's `BlockBuilder` API for block proposal and `BasicBlockExecutor` for block verification. The core insight is that **propose and verify use different reth APIs**: propose builds incrementally (transaction-by-transaction), while verify executes a complete block in batch.

### [PROPOSED] Execution Strategy

**Propose path** (block builder pattern):
1. Fetch raw tx bytes from `TxSource::pending()`
2. Decode via `TransactionSigned::decode_2718()`, recover senders
3. Create `reth_revm::State<DB>` wrapper around `InMemoryStateDb`
4. Call `evm_config.builder_for_next_block(&mut state, &parent_header, attrs)`
5. Call `builder.apply_pre_execution_changes()`
6. For each transaction: `builder.execute_transaction(recovered_tx)` — skip failures, continue
7. Extract `BundleState` from `State<DB>` via `state.take_bundle()`
8. Commit `BundleState` to `InMemoryStateDb`
9. Compute `state_root()`, `tx_root` (from executed tx list), `receipts_root` (from receipts)
10. Assemble and return `EvmBlock`

**Verify path** (batch executor pattern):
1. Decode transactions from `block.transactions`, recover senders
2. Reconstruct a `RecoveredBlock` matching the received block
3. Create `reth_revm::State<DB>` wrapper (on a SNAPSHOT, not canonical state)
4. Execute via `BasicBlockExecutor::execute_one(&recovered_block)`
5. Compare computed `state_root`, `tx_root`, `receipts_root`, `gas_used` against block fields
6. Do NOT commit state — verifier must not mutate canonical state

### [PROPOSED] State Snapshot Strategy

**Problem**: `propose()` must commit state to compute `state_root`, but if consensus rejects the proposed block, state is corrupted. There is no finalization callback in the `ConsensusApp` trait (Grounded: `crates/consensus/src/app.rs::ConsensusApp` has only genesis/propose/verify).

**Proposed solution — clone-based snapshots**:
- Before execution, clone `InMemoryStateDb` (it's HashMap-based, cloneable)
- Execute on the clone
- If propose succeeds AND block is later finalized: swap clone into canonical slot
- If propose fails or block rejected: discard clone

This is O(n) in state size but acceptable for in-memory MVP. A more efficient copy-on-write or journaling approach is deferred.

**Alternative considered**: Arc-based immutable snapshots. More complex, better perf. Deferred.

## Key Decisions

| ID | Decision | Rationale | Status |
|---|---|---|---|
| D-1 | Use reth `BlockBuilder` for propose, `BasicBlockExecutor` for verify | Builder gives incremental tx-by-tx control; executor gives batch re-execution for verification | [PROPOSED] |
| D-2 | Clone-based state snapshots for propose rollback safety | Simple, correct, acceptable perf for in-memory MVP | [PROPOSED] |
| D-3 | Verify does NOT commit state | Verifier must not mutate canonical state; only proposer commits | [PROPOSED] |
| D-4 | Skip invalid transactions during propose (don't fail the block) | Matches Ethereum behavior — invalid txs are excluded, valid ones execute | [PROPOSED] |
| D-5 | `NextBlockEnvAttributes` uses defaults for Sahara chain | `suggested_fee_recipient` = zero address, `prev_randao` = zero, `parent_beacon_block_root` = None, `withdrawals` = empty | [PROPOSED] |
| D-6 | Compute `tx_root` and `receipts_root` using alloy-trie | Matches Ethereum standard; `alloy_trie` already a dependency | [PROPOSED] |

## Risk Areas

| Risk | Impact | Mitigation |
|---|---|---|
| reth API instability (vendored code may change) | High — execution flow breaks | Pin vendor commit; wrap reth calls in thin adapter layer |
| State snapshot clone is O(n) state size | Medium — performance on large states | Acceptable for MVP; deferred COW optimization |
| `InMemoryStateDb::commit()` may miss BundleState edge cases | Medium — state corruption | Thorough unit tests on commit with complex BundleState inputs |
| No finalization callback means propose commits speculatively | High — state inconsistency on rejected blocks | Clone-based snapshot (D-2) |
| `builder.finish()` expects `StateRootProvider` impl | Medium — compilation error | Bypass reth's state_root computation; compute our own via `InMemoryStateDb::state_root()` after commit |
| Transaction decoding/recovery failures | Low — individual tx failure | Skip invalid txs (D-4); log warnings |

## Implementation Ordering

1. **Transaction decode/recover helper** — Pure function, no state dependency. Testable in isolation.
2. **Propose execution flow** — Wire reth BlockBuilder into `propose()`. Requires (1).
3. **State snapshot mechanism** — Add Clone to InMemoryStateDb (or wrapper). Integrate into propose.
4. **Verify re-execution flow** — Wire reth BasicBlockExecutor into `verify()`. Requires (1).
5. **Block field computation** — `tx_root`, `receipts_root` via alloy-trie. Requires (2).
6. **Integration tests** — End-to-end propose→verify cycle with real transactions.

## Strategy Triage

| Open Question | Classification | Action |
|---|---|---|
| `builder.finish()` requires `StateRootProvider` — does InMemoryStateDb implement it? | `information-gap` | Inspect vendored reth `StateRootProvider` trait; likely need to bypass and compute our own |
| Should invalid txs in propose be silently skipped or cause block failure? | `decision-gap` | Proposed: skip (D-4). Need confirmation. |
| Exact `NextBlockEnvAttributes` values for Sahara chain | `information-gap` | Proposed defaults (D-5). Acceptable for MVP. |
| State snapshot approach (clone vs COW vs journal) | `decision-gap` | Proposed: clone (D-2). Need confirmation for MVP acceptance. |
