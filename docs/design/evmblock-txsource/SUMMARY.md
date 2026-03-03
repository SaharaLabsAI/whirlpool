# SUMMARY — EvmBlock TxSource

## What

Implement `InMemoryTxPool` — a minimal, thread-safe, in-memory transaction pool that replaces the current `NoopTxSource` in whirlpool-node. This is **Sub-Intent 2** of the evm-tx-execution design.

## Why

The EVM execution engine (Sub-Intent 1) is fully implemented but receives no transactions because `NoopTxSource` always returns an empty vec. A real `TxSource` implementation is needed so that `EvmApplication::propose()` can include actual transactions in blocks.

## Design

**One new struct** in the `app` crate:

```rust
pub struct InMemoryTxPool {
    txs: Mutex<Vec<Vec<u8>>>,
}
```

- `push(tx: Vec<u8>)` — adds raw EIP-2718 encoded transaction bytes to the buffer
- `pending() -> Vec<Vec<u8>>` — drains and returns all buffered transactions (implements `TxSource`)
- Thread-safe via `Mutex` — usable as `Arc<dyn TxSource + Send + Sync>`

**No new crates. No new dependencies. No trait changes.**

## Key Decisions

1. **Drain semantics**: `pending()` clears the buffer — each tx included in at most one block proposal
2. **No validation**: pool stores raw bytes; the executor's `decode_transactions` handles invalid txs
3. **Mutex over RwLock**: both `push` and `pending` are write ops — Mutex is simpler and sufficient
4. **Located in `app` crate**: co-located with the `TxSource` trait definition

## Changes

| Crate | File | Change |
|---|---|---|
| `app` | `src/traits.rs` | Add `InMemoryTxPool` struct + `TxSource` impl + unit tests |
| `app` | `src/lib.rs` | Add `InMemoryTxPool` to re-exports |
| `whirlpool-node` | `src/main.rs` | Swap `NoopTxSource` → `InMemoryTxPool` |
| `app-evm` | `tests/integration.rs` | Add integration test using `InMemoryTxPool` |

## Implementation Order

1. `InMemoryTxPool` struct + TxSource impl (app)
2. Unit tests (app)
3. Node wiring update (whirlpool-node)
4. Integration test (app-evm)

## Risks & Limitations

- **Txs lost on propose failure** — acceptable for MVP; re-insertion is future work
- **No ordering/priority** — FIFO only; gas-price sorting is future work
- **No pool limits** — unbounded; eviction policies are future work

## Prior Design

Continues `docs/design/evm-tx-execution/` — see that design for the execution engine context.
