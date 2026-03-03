# FLOWS — EvmBlock TxSource

## F1: Transaction Submission

```
caller
  │
  ▼
InMemoryTxPool::push(raw_tx: Vec<u8>)
  │ lock self.txs (Mutex)
  │ self.txs.push(raw_tx)
  │ unlock
  ▼
(returns)
```

**Error path**: `push()` is infallible. Lock poisoning would panic (acceptable for MVP — indicates a prior panic in the same lock scope, which is unrecoverable).

## F2: Transaction Consumption (existing propose flow, updated source)

```
EvmApplication::propose(parent, height)
  │
  ├─ self.tx_source.pending()          ← InMemoryTxPool drains buffer
  │    │ lock self.txs
  │    │ std::mem::take(&mut *guard)   ← atomic drain
  │    │ unlock
  │    │ return Vec<Vec<u8>>
  │
  ├─ decode_transactions(raw_txs)      ← filter_map, skip invalid
  ├─ BlockBuilder execute each tx
  ├─ commit BundleState to state_db
  ├─ compute roots (state, tx, receipt)
  └─ return (EvmBlock, ExecutionResult)
```

**Error path**: If `pending()` returns empty vec, propose builds an empty block (existing behavior with NoopTxSource). If some txs are invalid, `decode_transactions` skips them silently.

## F3: Node Wiring

```
main()
  │
  ├─ let tx_pool = Arc::new(InMemoryTxPool::new())
  ├─ let app = EvmApplication::new(config, state_db, tx_pool.clone())
  ├─ let adapter = ApplicationAdapter::new(app)
  │   ... (consensus wiring unchanged)
  │
  └─ tx_pool retained for future RPC submission endpoint
```

## Implementation Slices

| Slice | Description | Files | Status |
|---|---|---|---|
| S-1 | `InMemoryTxPool` struct + `TxSource` impl | `crates/app/src/traits.rs`, `crates/app/src/lib.rs` | [PROPOSED] |
| S-2 | Unit tests (push, pending, drain, thread-safety) | `crates/app/src/traits.rs` (tests module) | [PROPOSED] |
| S-3 | Node wiring update | `crates/whirlpool-node/src/main.rs` | [PROPOSED] |
| S-4 | Integration test with EvmApplication | `crates/app-evm/tests/integration.rs` | [PROPOSED] |
