# INTENT

## Original Statement
Add persistent storage to the mempool so transactions survive node restarts.

## Parsed Intent
Replace the in-memory transaction pool (`InMemoryTxPool`) with a persistent storage-backed implementation that preserves pending transactions across node restarts while maintaining the existing `TxSource` trait contract.

## Requirements
1. **Persistence**: Transactions must survive node process restarts
2. **Trait compatibility**: Must implement the existing `TxSource` trait (`fn pending(&self) -> Vec<Vec<u8>>`)
3. **Push semantics**: Must support `push(tx: Vec<u8>)` for adding new transactions
4. **Drain semantics**: `pending()` must drain the buffer (FIFO order), consistent with `InMemoryTxPool`
5. **Concurrency**: Must be safe for concurrent access (multi-threaded runtime)
6. **Integration**: Must plug into `whirlpool-node` main.rs and `EthRpcContext` with minimal wiring changes
7. **Performance**: Persistence must not significantly degrade transaction submission latency

## Depth
module

## Focus Crates
- `app` — TxSource trait + InMemoryTxPool (primary change target)
- `whirlpool-node` — Node wiring (integration point)
- `rpc-eth` — EthRpcContext tx_pool field (may need type change)
- `state-reth` — Reference for existing MDBX persistence patterns

## Crate Boundaries
- **app**: Owns TxSource trait. Trait will be extended with `push()` method.
- **mempool** [NEW]: New crate for persistent mempool implementation. Depends on `app` for trait, uses raw `libmdbx-rs` for storage.
- **whirlpool-node**: Wiring only — instantiates `PersistentTxPool` and passes to consumers as trait object.
- **rpc-eth**: Consumer of tx_pool — generified to accept `Arc<dyn TxSource>` instead of concrete `InMemoryTxPool`.
- **state-reth**: Reference only — not modified, but patterns (MDBX usage) inform design.
- **app-evm**: No changes required — already uses `Arc<dyn TxSource + Send + Sync>`.

## Design Refinements (from STRATEGY)

### MVP Scope Decisions
1. **Drain-on-pending semantics preserved**: Crash between propose and finalize → txs lost (same as today). Future enhancement: lifecycle tracking (`submitted` → `proposed` → `finalized`).
2. **No deduplication**: InMemoryTxPool stores duplicates; PersistentTxPool will too (auto-increment keys). Future enhancement: add tx-hash index for natural dedup.
3. **Storage backend**: Raw `libmdbx-rs` (not reth-db) to avoid vendor table enum coupling. Separate MDBX database directory for mempool.
4. **Key strategy**: Auto-increment u64 preserves FIFO ordering, no decoding/hashing overhead on push.
5. **Trait extension**: Add `push()` to `TxSource` trait (in-tree breaking change, all implementors updated atomically).
6. **EthRpcContext generification**: Use trait object (`Arc<dyn TxSource>`) to avoid type parameter propagation.

### Implementation Constraints
- **No vendor modification**: Cannot extend reth-db `Tables` enum.
- **FIFO ordering**: Consensus expects oldest-first drain from `pending()`.
- **Concurrent safety**: MDBX provides concurrent readers + single writer; Rust `Mutex` guards write operations.
- **Path isolation**: Mempool DB at `{persistent_storage_dir}/mempool`, separate from state/block storage.

### Out of Scope (Future Enhancements)
- Transaction lifecycle tracking (proposed → finalized, re-queue on crash)
- Transaction deduplication (tx-hash index)
- Performance benchmarking (mempool ops not hot path)
- Async TxSource trait (current callers are sync)

## Non-Goals
- Transaction validation logic changes
- Consensus/block proposal changes
- P2P transaction propagation
- State persistence changes (already handled by state-reth)
- reth-db modifications (vendor code frozen)
