# Shared Flows Index — evmblock-txsource

## F1: Transaction Submission Flow

```
External caller → InMemoryTxPool::push(raw_tx: Vec<u8>)
  → lock internal buffer
  → append tx bytes
  → unlock
```

## F2: Transaction Consumption Flow (existing, unchanged)

```
EvmApplication::propose()
  → self.tx_source.pending()
    → InMemoryTxPool drains buffer, returns Vec<Vec<u8>>
  → decode_transactions (filter_map, skip invalid)
  → execute via BlockBuilder
```

## F3: Node Wiring Flow

```
main()
  → InMemoryTxPool::new()
  → Arc::new(tx_pool)
  → EvmApplication::new(config, state_db, tx_pool.clone())
  → retain tx_pool handle for future RPC wiring
```

## Key Design Decision: Drain vs Clone

`pending()` SHOULD drain the buffer (take + clear) rather than clone, because:
- Each tx should be included in at most one proposed block
- If propose fails, txs are lost (acceptable for MVP; retry logic is future work)
- Simpler implementation
