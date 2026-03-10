# Summary — EVM Transaction Execution

## What

Replace the empty-block stubs in `app-evm::EvmApplication::propose()` and `verify()` with real EVM transaction execution. After this work, `propose()` will fetch transactions from a `TxSource`, execute them via reth's EVM, commit state changes to `InMemoryStateDb`, and return an `EvmBlock` with correct `state_root`, `tx_root`, `receipts_root`, and `gas_used`. `verify()` will re-execute all transactions from a received block and validate that the computed results match.

This is **Sub-Intent 1** of "produce EVM block for whirlpool-node", focusing on the execution engine. Transaction sourcing (Sub-Intent 2) and node wiring (Sub-Intent 3) follow.

## Why

The current `propose()` returns empty blocks with hardcoded `EMPTY_ROOT_HASH` values — no transactions are executed, and the EVM is effectively unused. This design bridges the gap between the existing consensus infrastructure and real EVM execution.

## How (Architecture)

**Two reth API patterns**, one for each path:

- **Propose** uses reth's `BlockBuilder` — incremental, transaction-by-transaction execution. This allows skipping invalid transactions and building the block as we go.
- **Verify** uses reth's `BasicBlockExecutor` — batch execution of a complete block. Deterministic re-execution for validation.

**State management**: Clone-based snapshots. Before execution, clone `InMemoryStateDb`. Execute on the clone. Commit to canonical only on propose success. Verify never commits to canonical state.

**Block field computation**: `tx_root` and `receipts_root` computed via `alloy-trie` (Ethereum-standard trie roots). `state_root` computed via existing `InMemoryStateDb::state_root()` (flat keccak256, not MPT — acceptable for MVP).

## Scope

| Crate | Changes |
|---|---|
| `app-evm` | Replace propose/verify stubs with real EVM execution |
| `state` | Verify commit correctness; add Clone for snapshots |
| `app` | No changes (stable traits) |
| Others | No changes |

## Key Decisions

1. **D-1**: BlockBuilder for propose, BasicBlockExecutor for verify
2. **D-2**: Clone-based state snapshots (simple, correct for MVP)
3. **D-3**: Verify does NOT commit to canonical state
4. **D-4**: Skip invalid transactions during propose
5. **D-5**: Default `NextBlockEnvAttributes` for Sahara chain
6. **D-6**: alloy-trie for tx_root/receipts_root computation

## Open Items

- **B-1**: reth's `builder.finish()` expects `StateRootProvider` — bypass with custom finish flow
- **B-2**: No finalization callback — acceptable for single-proposer MVP
- **B-3/B-4**: Decision confirmations needed on tx skip behavior and snapshot approach

## Implementation Order

1. Transaction decode/recover helper
2. Propose execution flow (reth BlockBuilder)
3. State snapshot mechanism (Clone)
4. Verify re-execution flow (reth BasicBlockExecutor)
5. Block field computation (alloy-trie)
6. Integration tests (propose→verify round-trip)
