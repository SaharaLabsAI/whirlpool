# Shared Domain Map — evmblock-txsource

## Domain: Transaction Sourcing

**Owner crate**: `app`
**Boundary**: `TxSource` trait (already exists)

### Entities

- `InMemoryTxPool` [PROPOSED] — thread-safe in-memory buffer of raw transaction bytes
  - Internal: `Mutex<Vec<Vec<u8>>>` or `RwLock<Vec<Vec<u8>>>`
  - Public API: `new()`, `push(tx: Vec<u8>)`, `TxSource::pending()`

### Invariants

1. `pending()` returns all buffered txs and drains the buffer (propose consumes)
2. `push()` is callable from any thread (Send + Sync)
3. No validation — raw bytes passed through (validation happens in executor decode step)
4. Order preserved (FIFO)

### Cross-domain boundary

- **Upstream**: External caller → `InMemoryTxPool::push()`
- **Downstream**: `EvmApplication::propose()` → `TxSource::pending()`
