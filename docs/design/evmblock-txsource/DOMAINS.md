# DOMAINS — EvmBlock TxSource

## Domain 1: Transaction Sourcing

**Owner crate**: `app`
**Boundary trait**: `TxSource` (grounded — `crates/app/src/traits.rs:23`)

### Entities

| Entity | Status | Location |
|---|---|---|
| `TxSource` trait | Grounded | `crates/app/src/traits.rs:23-25` |
| `NoopTxSource` | Grounded | `crates/app/src/traits.rs:27-33` |
| `InMemoryTxPool` | [PROPOSED] | `crates/app/src/traits.rs` (new) |

### [PROPOSED] `InMemoryTxPool` Contract

```rust
pub struct InMemoryTxPool {
    txs: Mutex<Vec<Vec<u8>>>,
}

impl InMemoryTxPool {
    /// Create an empty transaction pool.
    pub fn new() -> Self;

    /// Add a raw EIP-2718 encoded transaction to the pool.
    pub fn push(&self, tx: Vec<u8>);
}

impl TxSource for InMemoryTxPool {
    /// Drain and return all pending transactions.
    /// After this call, the pool is empty.
    fn pending(&self) -> Vec<Vec<u8>>;
}
```

### Invariants

1. **Drain semantics**: `pending()` returns all buffered txs and clears the buffer atomically (under lock)
2. **Thread safety**: `InMemoryTxPool` is `Send + Sync` (Mutex provides this)
3. **No validation**: raw bytes stored as-is; validation is executor's responsibility
4. **FIFO order**: transactions returned in insertion order
5. **Idempotent empty**: `pending()` on empty pool returns `Vec::new()`

## Domain 2: EVM Execution (unchanged)

**Owner crate**: `app-evm`
No changes to this domain. `EvmApplication` already consumes `Arc<dyn TxSource + Send + Sync>`.

## Wiring

| Source | Target | Mechanism | Change |
|---|---|---|---|
| External caller | `InMemoryTxPool::push()` | Direct method call | [PROPOSED] — new |
| `InMemoryTxPool::pending()` | `EvmApplication::propose()` | `TxSource` trait (dyn dispatch) | Wire update only |
| `whirlpool-node::main()` | `EvmApplication::new()` | Constructor arg | Swap `NoopTxSource` → `InMemoryTxPool` |

## Boundary Contract

The cross-domain boundary is the existing `TxSource` trait. No changes to the trait itself. The new `InMemoryTxPool` conforms to the existing contract exactly.
