# Intent — EVM Transaction Execution

## Objective

Implement real EVM transaction execution in `app-evm` so that `propose()` and `verify()` execute transactions via reth's block executor, commit state changes to `state::InMemoryStateDb`, and produce correct `state_root`, `tx_root`, `receipts_root`, and `gas_used` in `app::EvmBlock`.

This is **Sub-Intent 1** of the broader "produce EVM block for whirlpool-node" design. It focuses on the execution engine and state commit layer, deferring transaction sourcing and node wiring to subsequent sub-intents.

## Scope

### In-Scope Crates

| Crate | Role |
|---|---|
| `app-evm` | EVM configuration, block executor wiring, propose/verify implementation |
| `state` | In-memory state DB, BundleState commit, state root computation |

### In-Scope Changes

1. **`app-evm::EvmApplication::propose()`** — Replace empty-block stub with real EVM transaction execution using reth's `BlockBuilder` API (`crates/app-evm/src/executor.rs::EvmApplication::propose`)
2. **`app-evm::EvmApplication::verify()`** — Replace state-root-only check with full transaction re-execution and result comparison (`crates/app-evm/src/executor.rs::EvmApplication::verify`)
3. **Transaction decoding** — Decode raw `Vec<u8>` transaction bytes via `TransactionSigned::decode_2718` and recover senders
4. **State commit orchestration** — Ensure `state::InMemoryStateDb::commit()` is called with the `BundleState` produced by EVM execution
5. **Block field computation** — Compute correct `tx_root`, `receipts_root`, `gas_used` from execution results

### Out-of-Scope

- **Transaction source** — `TxSource` implementation (Sub-Intent 2). This design assumes `NoopTxSource` or any impl providing `Vec<Vec<u8>>`.
- **Node wiring** — Changes to `whirlpool-node/main.rs` (Sub-Intent 3)
- **Consensus / P2P** — No changes to `consensus`, `consensus-simplex`, `p2p`, `p2p-commonware`
- **Merkle Patricia Trie state root** — Flat keccak256 state root remains (current `state::InMemoryStateDb::state_root()`)
- **Disk persistence** — In-memory only
- **JSON-RPC / Engine API** — Not in scope
- **Gas pricing / MEV** — Not in scope
- **State snapshot/rollback** — Identified as BLOCKER; a minimal solution is proposed but full snapshot/restore is deferred

## Success Criteria

1. **SC-1**: `propose()` fetches transactions from `TxSource`, decodes them, executes via reth EVM, commits state, and returns an `EvmBlock` with correct `tx_root`, `receipts_root`, `gas_used`, `state_root`
2. **SC-2**: `verify()` re-executes all transactions from a received block and validates that computed `state_root`, `tx_root`, `receipts_root`, `gas_used` match the block's fields
3. **SC-3**: `InMemoryStateDb::commit()` correctly processes the `BundleState` from EVM execution (account changes, storage, bytecodes)
4. **SC-4**: Invalid transactions are handled gracefully (skipped or error propagated, per design decision)
5. **SC-5**: State is not permanently corrupted if a proposed block is later rejected by consensus (requires at minimum a clone-based snapshot approach)
6. **SC-6**: All existing tests continue to pass; new unit and integration tests cover the execution path

## Assumptions

- The `Application` trait's async signature is stable and will not change (`crates/app/src/traits.rs::Application`)
- `EvmBlock` fields are sufficient for EVM execution results (no new fields needed) (`crates/app/src/types.rs::EvmBlock`)
- Reth's `ConfigureEvm` / `BlockBuilder` API in the vendored code is the correct integration point (`vendor/reth/crates/evm/evm/src/execute.rs`)
- `InMemoryStateDb` already handles the core BundleState commit correctly (`crates/state/src/db.rs::InMemoryStateDb::commit`)
- Chain spec (Sahara, chain ID 313371, 30M gas, Cancun) is fixed (`crates/app-evm/src/config.rs::build_sahara_chain_spec`)
