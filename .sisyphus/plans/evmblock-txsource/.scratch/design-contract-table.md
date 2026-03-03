# Design Contract Table — EvmBlock TxSource

## Scope Boundaries
- **In-scope**: `InMemoryTxPool` struct in `app` crate, node wiring update in `whirlpool-node`, unit tests (push/pending/drain), integration test (propose with pool).
- **Out-of-scope**: JSON-RPC endpoint, tx validation, tx ordering/priority, pool size limits, duplicate detection, P2P gossip, tx re-insertion.

## Crate Ownership
| Crate | Capability |
|---|---|
| `app` | `InMemoryTxPool` (storage + `TxSource` impl) |
| `whirlpool-node` | Node wiring (dependency injection) |
| `app-evm` | Integration testing (EVM + Pool) |

## Public Interfaces
### `app` crate (`traits.rs`)
```rust
pub struct InMemoryTxPool { txs: Mutex<Vec<Vec<u8>>> }
impl InMemoryTxPool {
    pub fn new() -> Self;
    pub fn push(&self, tx: Vec<u8>);
}
impl TxSource for InMemoryTxPool {
    fn pending(&self) -> Vec<Vec<u8>>; // Drains buffer
}
```

## Flow Requirements
- **F1 (Submission)**: `push(raw_tx)` -> acquire lock -> append to `txs` -> release.
- **F2 (Consumption)**: `propose()` -> `pending()` -> acquire lock -> `std::mem::take` (drain) -> release -> return.
- **F3 (Wiring)**: `main()` -> create `Arc<InMemoryTxPool>` -> inject into `EvmApplication`.

## Implementation Slices
- **S-1**: `InMemoryTxPool` implementation in `app` crate.
- **S-2**: Unit tests for pool semantics and thread-safety.
- **S-3**: Node wiring update in `whirlpool-node`.
- **S-4**: Integration test in `app-evm`.

## Test Contracts
- **T-1**: `new_pool_is_empty` — `new()` returns empty `pending()`.
- **T-2**: `push_single_tx` — 1 push = 1 pending tx.
- **T-3**: `push_multiple_txs` — N pushes = N pending txs (FIFO).
- **T-4**: `pending_drains_buffer` — calling `pending()` twice returns empty second time.
- **T-5**: `push_after_drain` — ensure buffer reset works correctly.
- **T-6**: `concurrent_push` — thread-safety check.
- **T-7**: `propose_with_in_memory_pool` — integration: push -> propose -> block contains tx.

## Active Blockers
- None.

## Key Decisions
- **D-1**: Place `InMemoryTxPool` in `app` crate.
- **D-2**: Use `Mutex<Vec<Vec<u8>>>` for simple thread-safe writes.
- **D-3**: `pending()` uses `std::mem::take` for atomic drain.
- **D-4**: Keep `NoopTxSource` for test compatibility.
- **D-5**: Retain `Arc<InMemoryTxPool>` handle in node `main`.

## Success Criteria
- **SC-1**: `InMemoryTxPool` implements `TxSource`.
- **SC-2**: `push()` adds raw tx bytes.
- **SC-3**: `pending()` returns and drains buffer.
- **SC-4**: Thread-safe (Send + Sync).
- **SC-5**: Node wiring updated to use new pool.
- **SC-6**: Existing tests pass.
- **SC-7**: Integration: push -> propose includes tx in block.
