# Task 01: InMemoryTxPool implementation + unit tests

## Summary
Implement the `InMemoryTxPool` struct in the `app` crate and its corresponding `TxSource` trait implementation. This provides a thread-safe buffer for transactions that is drained when `pending()` is called.

## Crate(s)
- `app` (primary)

## Files Changed
- `crates/app/src/traits.rs` — Implementation of `InMemoryTxPool` and `TxSource`.
- `crates/app/src/lib.rs` — Re-export of `InMemoryTxPool`.

## Dependencies
- None

## Design Refs
- `docs/design/evmblock-txsource/FLOWS.md S-1, S-2`
- `docs/design/evmblock-txsource/TESTS.md T-1, T-2, T-3, T-4, T-5, T-6`

## TDD Sequence
1. **Red**: Write unit tests `new_pool_is_empty`, `push_single_tx`, `push_multiple_txs_fifo_order`, `pending_drains_buffer`, `push_after_drain`, and `concurrent_push` in `traits.rs`.
2. **Green**: Implement `InMemoryTxPool` with `Mutex<Vec<Vec<u8>>>`, `new()`, `push()`, and `TxSource::pending()` using `std::mem::take`.
3. **Verify**: Run AC commands to ensure all tests pass.

## Implementation Details
Use a `Mutex<Vec<Vec<u8>>>` to store pending transactions. The `pending()` method must acquire the lock and use `std::mem::take` to return the current buffer while leaving an empty one in its place. Ensure the `app` crate re-exports the pool so it can be used by the node.

## Acceptance Criteria
```
nix develop --command cargo test -p app -- traits
```

## Evidence
- `.sisyphus/evidence/evmblock-txsource/01-impl-and-unit-tests.log`
