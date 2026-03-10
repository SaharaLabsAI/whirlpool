# STRATEGY — EvmBlock TxSource

## Architecture Direction

The `TxSource` trait already exists with a clean, minimal interface (`pending() -> Vec<Vec<u8>>`). The executor (`EvmApplication`) already consumes it via dynamic dispatch (`Arc<dyn TxSource + Send + Sync>`). This design adds only a concrete implementation — no trait changes, no new crate, no architectural shifts.

### [PROPOSED] `InMemoryTxPool`

A `Mutex<Vec<Vec<u8>>>` behind a struct that implements `TxSource`. The `pending()` call takes the lock, drains the buffer (via `std::mem::take`), and returns the contents. A `push(tx: Vec<u8>)` method takes the lock and appends.

**Why Mutex, not RwLock**: Both `push()` and `pending()` are write operations (append / drain). A `RwLock` provides no benefit since there are no read-only consumers. `Mutex` is simpler and sufficient.

**Why drain, not clone**: Each transaction should be included in at most one proposed block. Drain ensures this. If a proposal fails, transactions are lost — acceptable for MVP per assumption A1.

## Key Decisions

| # | Decision | Rationale |
|---|---|---|
| D-1 | Place `InMemoryTxPool` in `app` crate alongside `TxSource` trait | Co-location with trait; no new crate needed |
| D-2 | Use `Mutex<Vec<Vec<u8>>>` for interior mutability | Simple, correct; both operations are writes |
| D-3 | `pending()` drains via `std::mem::take` | Ensures tx-per-block uniqueness; simpler than clone |
| D-4 | Keep `NoopTxSource` unchanged | Backward compatibility for tests |
| D-5 | Retain `Arc<InMemoryTxPool>` handle in node main | Allows future RPC wiring without refactoring |

## Risk Areas

| Risk | Impact | Mitigation |
|---|---|---|
| Txs lost on propose failure | Low (MVP) | Document as known limitation; future: re-insert |
| Lock contention under high tx rate | Low (single-node MVP) | Future: crossbeam channel or lock-free queue |
| No validation = invalid txs in pool | None — executor already skips invalid | Existing `decode_transactions` filter_map handles this |

## Implementation Ordering

| Step | Description | Crate | Depends on |
|---|---|---|---|
| S-1 | Implement `InMemoryTxPool` with push + TxSource | `app` | — |
| S-2 | Unit tests for push/pending/drain/thread-safety | `app` | S-1 |
| S-3 | Update node wiring to use `InMemoryTxPool` | `whirlpool-node` | S-1 |
| S-4 | Integration test: push → propose → block contains tx | `app-evm` | S-1 |

## Strategy Triage

| Open question | Classification | Resolution |
|---|---|---|
| (none) | — | All scope clear from prior design |

No blockers. Proceeding to build.
