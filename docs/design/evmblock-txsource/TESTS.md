# TESTS — EvmBlock TxSource

## Unit Tests (app crate)

| # | Test | Verifies | Boundary |
|---|---|---|---|
| T-1 | `new_pool_is_empty` | `InMemoryTxPool::new()` → `pending()` returns empty vec | Real |
| T-2 | `push_single_tx` | Push 1 tx → `pending()` returns vec with that tx | Real |
| T-3 | `push_multiple_txs` | Push N txs → `pending()` returns all N in FIFO order | Real |
| T-4 | `pending_drains_buffer` | `pending()` → second `pending()` returns empty | Real |
| T-5 | `push_after_drain` | `pending()` → `push()` → `pending()` returns new tx only | Real |
| T-6 | `concurrent_push` | Spawn N threads each pushing → `pending()` returns all N txs | Real |

## Integration Tests (app-evm crate)

| # | Test | Verifies | Boundary |
|---|---|---|---|
| T-7 | `propose_with_in_memory_pool` | Push signed tx → propose → block contains tx, state updated | Real pool, real EVM |

### T-7 Detail

```
1. Create InMemoryTxPool
2. Create funded account (via genesis)
3. Encode signed legacy transfer as EIP-2718 bytes
4. pool.push(encoded_tx)
5. EvmApplication::propose(genesis_block, 1)
6. Assert: block.transactions.len() == 1
7. Assert: block.gas_used > 0
8. Assert: recipient balance updated
```

This mirrors the existing `test_propose_and_verify_with_transfer` integration test but uses `InMemoryTxPool` instead of `MockTxSource`.

## Cross-Crate Seams

| Seam | Unit tests | Integration tests |
|---|---|---|
| `TxSource::pending()` | Real `InMemoryTxPool` | Real `InMemoryTxPool` |
| `EvmApplication` | Not tested (app crate) | Real |
| State DB | Not involved | Real `InMemoryStateDb` |

## Success Criteria Mapping

| Criterion | Test(s) |
|---|---|
| SC-1: Implements TxSource | T-1 through T-6 (compilation) |
| SC-2: push() adds txs | T-2, T-3 |
| SC-3: pending() drains | T-4, T-5 |
| SC-4: Thread-safe | T-6 |
| SC-5: Node wiring | Code review (S-3) |
| SC-6: Existing tests pass | `cargo test` |
| SC-7: Integration | T-7 |
