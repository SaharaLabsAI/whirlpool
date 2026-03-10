# app — Crate Contract (evmblock-txsource)

## 1. Purpose

Hosts the `TxSource` trait and its implementations. This design adds `InMemoryTxPool` — a thread-safe in-memory transaction buffer.

## 2. Public API Changes

### New: `InMemoryTxPool`

```rust
/// [PROPOSED] Minimal in-memory transaction pool.
pub struct InMemoryTxPool {
    txs: Mutex<Vec<Vec<u8>>>,  // private
}

impl InMemoryTxPool {
    pub fn new() -> Self;
    pub fn push(&self, tx: Vec<u8>);
}

impl TxSource for InMemoryTxPool {
    fn pending(&self) -> Vec<Vec<u8>>;
}
```

### Re-export in `lib.rs`

Add `InMemoryTxPool` to `pub use traits::...` line.

### Unchanged

- `TxSource` trait — no signature changes
- `NoopTxSource` — retained as-is

## 3. Dependencies

No new dependencies. Uses `std::sync::Mutex` only.

## 4. Key Behaviors

- `push()`: acquires lock, appends tx bytes, releases lock. Infallible.
- `pending()`: acquires lock, `std::mem::take` drains buffer, releases lock, returns drained vec.
- Empty pool: `pending()` returns `Vec::new()`.
- Lock poisoning: panics (standard Mutex behavior — acceptable since poison implies prior panic).

## 5. Test Surface

- T-1 through T-6: unit tests in `#[cfg(test)] mod tests` inside `traits.rs`
- See TESTS.md for full spec
