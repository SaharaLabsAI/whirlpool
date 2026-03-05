# Design Intent: EVM Block Production for whirlpool-node

## Intent

Enable `whirlpool-node` to produce **real EVM blocks** with transaction execution — replacing the current MVP that only produces empty blocks. This design covers the full block production lifecycle: transaction ingestion → EVM execution → state commitment → block assembly → consensus finalization, all orchestrated from `whirlpool-node`.

## Motivation

The current `whirlpool-node` binary wires `EvmApplication` → `ApplicationAdapter` → `CommonwareEngine` and successfully runs consensus, but `EvmApplication::propose()` only builds empty blocks (no transactions are processed). The `TxSource` trait exists but only `NoopTxSource` is implemented. The EVM execution pipeline (via reth `ConfigureEvm`) is configured but never invoked during block production.

To produce meaningful EVM blocks, we need:
1. A real transaction source (mempool or equivalent)
2. Actual EVM transaction execution within `propose()`
3. State commitment after execution (`InMemoryStateDb::commit()`)
4. Correct block assembly with execution results (state_root, receipts_root, gas_used, transactions)
5. Proper block verification in `verify()` that re-executes transactions and validates results

## Crates in scope

| Crate | Role | Status |
|---|---|---|
| `whirlpool-node` | Primary: node binary, wiring, block lifecycle orchestration | Grounded (exists, needs enhancement) |
| `app-evm` | Core: EVM execution, block proposal/verification | Grounded (exists, needs tx execution in propose/verify) |
| `app` | Interface: Application trait, TxSource, EvmBlock, ApplicationAdapter | Grounded (exists, may need TxSource enhancement) |
| `state` | Supporting: InMemoryStateDb, state commitment | Grounded (exists, commit() already implemented) |

## Crates referenced but out of scope

| Crate | Reason |
|---|---|
| `consensus` | Stable trait layer — no changes expected |
| `consensus-simplex` | Consensus engine — no changes expected |
| `p2p` / `p2p-commonware` | Network transport — no changes expected |

## Success criteria

1. **Transaction execution in propose()**: `EvmApplication::propose()` reads pending transactions from `TxSource`, executes them via the reth EVM pipeline, commits state changes, and assembles a complete `EvmBlock` with correct execution results.
2. **Transaction verification in verify()**: `EvmApplication::verify()` re-executes the block's transactions against parent state and validates that computed results match the block's claimed results (state_root, receipts_root, gas_used).
3. **TxSource implementation**: At least one concrete `TxSource` beyond `NoopTxSource` that provides transactions for block production (mempool or queue-based).
4. **State lifecycle**: State is correctly snapshotted before execution, committed after successful execution, and rolled back on failure.
5. **Block assembly correctness**: Produced `EvmBlock` contains valid transactions_root (MPT over tx list), receipts_root (MPT over receipts), correct gas_used, and accurate state_root.
6. **Wiring in whirlpool-node**: The binary correctly instantiates and connects the real TxSource, EVM execution pipeline, and state management.
7. **End-to-end flow**: Consensus triggers propose → transactions are executed → block is assembled → block is finalized → state is committed.

## Related design sessions

- `docs/design/evm-integration/` — covers `app`, `app-evm`, `state` crate contracts (rounds 1-3). This session builds on those contracts to deliver the actual block production flow.
- `docs/design/chain-binary/` — covers chain binary architecture.

## Out of scope

- MPT (Merkle Patricia Trie) for state root calculation (tracked as B-003 in evm-integration)
- State persistence / disk-backed storage (tracked as B-004 in evm-integration)
- JSON-RPC API for transaction submission
- P2P transaction propagation / gossip
- MEV / transaction ordering beyond simple FIFO
- Gas price / priority fee handling
- Account nonce management beyond basic sequential ordering
