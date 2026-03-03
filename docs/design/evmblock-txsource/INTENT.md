# INTENT — EvmBlock TxSource

## Objective

Implement a minimal in-memory transaction pool (`InMemoryTxPool`) that satisfies the `TxSource` trait, replacing `NoopTxSource` in the whirlpool-node binary. This is **Sub-Intent 2** of the evm-tx-execution design.

## Scope

### In-scope

1. **`InMemoryTxPool` struct** in `crates/app/src/traits.rs` — thread-safe buffer implementing `TxSource`
2. **Node wiring update** in `crates/whirlpool-node/src/main.rs` — swap `NoopTxSource` → `InMemoryTxPool`
3. **Unit tests** for push/pending/drain semantics
4. **Integration test** verifying `InMemoryTxPool` works with `EvmApplication::propose()`

### Out-of-scope

- JSON-RPC endpoint (`eth_sendRawTransaction`) — Sub-Intent 3
- Transaction validation before pool insertion (gas price, nonce, balance checks)
- Transaction ordering / priority (gas price sorting)
- Pool size limits / eviction policies
- Duplicate detection
- P2P transaction gossip
- Transaction re-insertion after failed proposals

## Success Criteria

| # | Criterion | Verification |
|---|---|---|
| SC-1 | `InMemoryTxPool` implements `TxSource` | Compiles, unit test |
| SC-2 | `push()` adds raw tx bytes to the pool | Unit test |
| SC-3 | `pending()` returns all buffered txs and drains the buffer | Unit test |
| SC-4 | Thread-safe — usable as `Arc<dyn TxSource + Send + Sync>` | Compiles, concurrent test |
| SC-5 | Node wiring uses `InMemoryTxPool` instead of `NoopTxSource` | Code review |
| SC-6 | Existing tests continue to pass | `cargo test` |
| SC-7 | Integration: push tx → propose includes it in block | Integration test |

## Assumptions

- A1: Simple drain semantics are acceptable (txs lost if propose fails)
- A2: No ordering guarantees beyond FIFO insertion order
- A3: The `TxSource` trait signature will not change (single `pending()` method)
- A4: `NoopTxSource` is retained for test convenience

## Prior Design Reference

Continues from `docs/design/evm-tx-execution/` where TxSource implementation was explicitly deferred.
