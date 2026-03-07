# Persistent Mempool Design Summary

## Executive Summary

This design adds crash-recoverable transaction persistence to Whirlpool's mempool by replacing the in-memory `InMemoryTxPool` with a new `PersistentTxPool` backed by an embedded MDBX database. Transactions submitted via RPC survive node restarts until they are proposed by consensus, eliminating the current loss window for pending transactions. The implementation preserves existing consensus semantics (FIFO drain-on-pending, no deduplication) while introducing zero changes to the execution layer.

**Core Achievement**: Reduces transaction loss window from "submission → finalization" to "drain → finalization" (matching current behavior for that window).

---

## Key Design Decisions

1. **Raw libmdbx-rs Storage Backend**
   - **Why**: reth-db's `Tables` enum cannot be extended without vendor modification. MDBX supports multiple independent databases via separate paths, allowing mempool persistence without coupling to reth-db abstractions.
   - **Benefit**: Zero vendor dependency changes, clean separation from state persistence.

2. **TxSource Trait Extension with `push()` Method**
   - **Why**: RPC layer currently uses concrete `Arc<InMemoryTxPool>` to access `push()`. Adding `push()` to trait enables polymorphic trait object usage (`Arc<dyn TxSource>`).
   - **Impact**: In-tree breaking change, all implementors updated atomically. Enables generification of `EthRpcContext` and future mempool variants.

3. **Auto-Increment u64 Keys for FIFO Ordering**
   - **Why**: Consensus expects oldest-first drain from `pending()`. Auto-increment keys preserve insertion order naturally via ascending key scan.
   - **Benefit**: Cheap inserts (no decoding/hashing), FIFO guaranteed, matches `InMemoryTxPool` semantics exactly.

4. **Drain-on-Pending Semantics Preserved**
   - **Why**: Consensus layer expects `pending()` to drain the pool (txs returned once, then removed). Changing this would require consensus modifications (out of scope).
   - **Implementation**: MDBX read-write transaction atomically scans all keys (ascending), collects values, deletes keys, commits.
   - **Crash Recovery**: If crash occurs before commit, transaction rolls back and txs remain in DB (recovered on restart).

5. **EthRpcContext Generification with Trait Object**
   - **Why**: Avoid type parameter propagation through RPC stack. Matches existing `EvmApplication` pattern (already uses `Arc<dyn TxSource>`).
   - **Benefit**: Minimal code changes, RPC layer stays agnostic to mempool implementation.

6. **New Dedicated Mempool Crate**
   - **Why**: Clean separation of concerns—mempool persistence is orthogonal to app-layer traits. Encapsulates MDBX dependency, enables isolated testing.
   - **Structure**: `mempool/src/` with `persistent.rs` (trait impl), `store.rs` (MDBX wrapper), `lib.rs` (exports).

7. **Accepted Risk: Transaction Loss Between Drain and Finalization**
   - **Why**: `pending()` drains DB before consensus finalization. If node crashes after drain but before finalization, txs are lost.
   - **Status**: This matches current `InMemoryTxPool` behavior—accepted for MVP. Future enhancement: add `proposed_txs` table to track lifecycle.

---

## Crates Affected

| Crate | Change Type | Summary |
|-------|-------------|---------|
| **mempool** (NEW) | Create | Persistent tx pool with MDBX backend, implements `TxSource` trait |
| **app** | Modify | Extend `TxSource` trait with `fn push(&self, tx: Vec<u8>)`, update existing implementors |
| **rpc-eth** | Modify | Change `EthRpcContext.tx_pool` from `Arc<InMemoryTxPool>` to `Arc<dyn TxSource + Send + Sync>` |
| **whirlpool-node** | Modify | Wire `PersistentTxPool::open(path)` instead of `InMemoryTxPool::new()`, compute mempool path |
| **app-evm** | Unchanged | Already uses `Arc<dyn TxSource>` trait object—transparent to persistence change |
| **state-reth** | Unchanged | Separate MDBX database directory, no interaction with mempool DB |
| **consensus-simplex** | Unchanged | No mempool interaction, wraps `ConsensusApp` callbacks only |

**Dependency Graph**:
```
mempool → app (TxSource trait)
         ↓
     libmdbx-rs

rpc-eth → app (trait object)
whirlpool-node → app, mempool, rpc-eth, app-evm
```

---

## Critical Flows

### 1. Transaction Submission (RPC → Mempool)
```
Client → eth_sendRawTransaction
      → rpc-eth: ctx.tx_pool.push(bytes)
      → mempool: PersistentTxPool::push(bytes)
            → Acquire mutex
            → next_id = counter.fetch_add(1)
            → MDBX: write_txn.put(next_id, bytes).commit()
            → Release mutex
      → Return tx_hash to client
```
**Durability**: After commit returns, tx survives crashes (ACID guarantee).

### 2. Transaction Drain (Consensus → Mempool)
```
Consensus → app-evm: EvmApplication.propose()
          → mempool: tx_source.pending()
                → Acquire mutex
                → MDBX: read_txn.iter(ascending) + delete_all + commit()
                → Release mutex
                → Return Vec<Vec<u8>> in FIFO order
          → Decode EIP-2718, execute in REVM
          → Build block proposal
```
**Atomicity**: Crash during drain (before commit) → txs remain in DB (recovered on restart).

### 3. Crash Recovery (Node Restart)
```
Node crash → restart
           → mempool: PersistentTxPool::open(path)
                 → Open existing MDBX database
                 → Scan for max key, resume counter
                 → Return pool (unproposed txs available)
           → Next propose() → drain recovers all persisted txs
```
**Recovered**: Txs committed but not yet drained.  
**Lost**: Txs drained (DB deleted) but not finalized (same as `InMemoryTxPool`).

---

## Implementation Roadmap

### Phase 1: Trait Foundation (Days 1-2)
- Extend `TxSource` trait with `push()` method (app crate)
- Update `InMemoryTxPool`, `NoopTxSource`, test mocks
- **Gate**: `cargo build -p app` succeeds, all tests pass

### Phase 2: RPC Generification (Day 3)
- Change `EthRpcContext` to accept trait object (rpc-eth crate)
- Update constructor signature, test helpers
- **Gate**: `cargo build -p rpc-eth` succeeds, existing RPC tests pass

### Phase 3: Persistent Implementation (Days 4-6)
- Create `mempool` crate skeleton
- Implement `MempoolStore` MDBX wrapper (store.rs)
- Implement `PersistentTxPool` with `TxSource` trait (persistent.rs)
- Write unit tests: push/drain, crash recovery, concurrency
- **Gate**: `cargo test -p mempool` passes, persistence verified

### Phase 4: Integration (Days 7-8)
- Wire `PersistentTxPool` in whirlpool-node main.rs
- Compute mempool path: `{persistent_storage_dir}/mempool`
- Integration tests: RPC → propose → restart → recover
- **Gate**: Full workspace build + test passes, end-to-end persistence works

**Estimated Duration**: 8-10 days (1 developer, includes testing)  
**Critical Path**: Phase 1 → Phase 2 → Phase 4 (Phases 2-3 can parallelize after Phase 1)

---

## Risks and Mitigations

### Risk 1: Transaction Loss Window (Drain → Finalize)
- **Impact**: Crash between `pending()` drain and finalization loses txs
- **Mitigation**: Accepted for MVP (matches current behavior), document clearly
- **Future**: Add `proposed_txs` table for full lifecycle tracking

### Risk 2: MDBX Performance Overhead
- **Impact**: Disk writes on every push could increase RPC latency
- **Mitigation**: MDBX is fast (<1ms expected), mempool ops not hot path
- **Validation**: Optional benchmark phase (Phase 5)

### Risk 3: Storage Path Misconfiguration
- **Impact**: Mempool DB overlapping state DB could cause corruption
- **Mitigation**: Dedicated subdir `{persistent_storage_dir}/mempool/`, path validation at startup
- **Status**: Low risk, enforced by implementation

### Risk 4: No Transaction Deduplication
- **Impact**: Duplicate tx submissions waste disk space
- **Mitigation**: Not a regression—`InMemoryTxPool` also stores duplicates
- **Future**: Add tx-hash index for optional dedup (post-MVP)

---

## Testing Strategy

### Unit Tests (Per-Crate)
- **app**: Trait extension compiles, implementors updated (UT-APP-01 to UT-APP-05)
- **mempool**: Push/drain, crash recovery, FIFO ordering, concurrency (UT-MEMPOOL-01 to UT-MEMPOOL-13)
- **rpc-eth**: Trait object acceptance, RPC methods unchanged (UT-RPC-01 to UT-RPC-04)
- **Total**: 28 unit test cases

### Integration Tests (Cross-Crate)
- **RPC → mempool**: TX persists to MDBX (INT-FLOW-01)
- **Mempool → consensus**: Drained txs execute in REVM (INT-FLOW-03, INT-FLOW-04)
- **Node wiring**: Fresh/existing DB startup (INT-WIRE-01, INT-WIRE-02)
- **Total**: 8 integration test cases

### Crash Recovery Tests
- **Crash after push, before drain**: TX recovered (INT-CR-01)
- **Crash during drain**: TX persists (uncommitted drain rolled back) (INT-CR-02)
- **Crash after drain, before finalize**: TX lost (accepted behavior) (INT-CR-03)
- **Total**: 4 crash recovery scenarios

### Property-Based Tests
- **FIFO ordering holds for N txs** (PROP-01)
- **Concurrent push preserves all txs** (PROP-02)
- **Drain is atomic (all-or-nothing)** (PROP-04)
- **Total**: 4 property tests

**Test Coverage**: 67 test cases map to INTENT success criteria (see TESTS.md)

---

## Success Criteria (from INTENT.md)

| Requirement | Implementation | Verification |
|-------------|----------------|--------------|
| Transactions survive node restarts | MDBX ACID guarantees, unproposed txs recovered on `open()` | INT-CR-01, INT-E2E-01 |
| Trait compatibility maintained | `TxSource` trait extended, all implementors updated | UT-APP-01, UT-RPC-01 |
| Push semantics functional | `TxSource::push()` trait method, MDBX write transaction | UT-MEMPOOL-02, UT-MEMPOOL-05 |
| Drain semantics preserved (FIFO) | Auto-increment keys, ascending scan on drain | UT-MEMPOOL-03, UT-MEMPOOL-06 |
| Concurrent access safe | `Mutex<MempoolStore>`, MDBX single-writer + concurrent readers | UT-MEMPOOL-07, UT-MEMPOOL-08 |
| whirlpool-node integration | `PersistentTxPool::open(path)` wiring, trait object injection | INT-WIRE-01, INT-WIRE-02 |
| Performance acceptable | MDBX <1ms overhead (expected, unmeasured), not hot path | Optional benchmark phase |

---

## Storage Schema

**Path**: `{persistent_storage_dir}/mempool/`

**Database**: MDBX unnamed database (default)

**Table**: `pending_txs` (key-value pairs)
- **Key**: `u64` (8 bytes, big-endian encoding for lexicographic order)
- **Value**: `Vec<u8>` (raw EIP-2718 encoded transaction bytes)

**Metadata**: Counter recovered from max key on startup (no separate metadata table)

**Example**:
```
Key: 0x0000000000000001 → Value: [0x02, 0xf8, 0x70, ...]  (EIP-1559 tx)
Key: 0x0000000000000002 → Value: [0x01, 0xf8, 0x65, ...]  (EIP-2930 tx)
Key: 0x0000000000000003 → Value: [0xf8, 0x6c, ...]        (Legacy tx)
```

**Operations**:
- **Push**: `next_id.fetch_add(1)` → `put(id.to_be_bytes(), tx)` → `commit()`
- **Drain**: `cursor.iter()` (ascending) → `collect()` → `cursor.del_all()` → `commit()`
- **Recovery**: Scan for max key, resume counter at `max_key + 1`

---

## Known Limitations (Accepted for MVP)

1. **No deduplication**: Duplicate txs stored with different keys (same as `InMemoryTxPool`)
2. **No lifecycle tracking**: Drain deletes from DB, no re-queue on crash before finalization
3. **No performance benchmarks**: MDBX overhead expected <1ms but unmeasured
4. **No transaction validation**: Invalid EIP-2718 txs accepted, validated later in `EvmApplication::propose()`
5. **No observability metrics**: No counters for push/drain errors, DB size, recovery count (future)

**All limitations match current `InMemoryTxPool` behavior or are deferred post-MVP enhancements.**

---

## Migration Path

### Development → Production
- **Fresh nodes**: Start with empty mempool DB, normal operation
- **Existing nodes**: In-memory `InMemoryTxPool` state lost on upgrade (acceptable—mempool is transient)
- **No data migration needed**: Mempool txs are ephemeral by design

### Rollback Strategy
- **If issues arise**: Change one line in `whirlpool-node/src/main.rs` back to `InMemoryTxPool::new()`
- **Remove dependency**: Delete `mempool = { path = "../mempool" }` from `whirlpool-node/Cargo.toml`
- **Persistent DB ignored**: Old mempool DB left on disk, no cleanup needed (or delete `mempool/` dir)

---

## Blockers and Warnings

### Active Blockers: 0

All design decisions resolved. Implementation ready to proceed.

### Warnings (Accepted Risks)

1. **Crash between propose and finalize loses transactions**
   - Status: Accepted MVP behavior, document clearly
   - Future: Implement lifecycle tracking (`submitted` → `proposed` → `finalized`)

2. **Storage path misconfiguration risk**
   - Mitigation: Enforce dedicated subdir, validate non-overlap
   - Future: Add path validation logic in `PersistentTxPool::open()`

3. **No deduplication remains efficiency risk**
   - Status: Not a regression (current behavior)
   - Future: Optional tx-hash index for dedup

---

## Next Steps

1. **Current Phase**: DESIGN (complete, awaiting PASS verdict)
2. **Next Phase**: PROVE—validate design against acceptance criteria via intent split
3. **Implementation**: Begin Phase 1 (Trait Foundation) after PROVE gate passed
4. **Target Completion**: 8-10 days from implementation start

---

## References

- **Full Design**: See `INDEX.md` for document inventory and reading guide
- **Detailed Flows**: `FLOWS.md` for sequence diagrams and error paths
- **Test Contracts**: `TESTS.md` for 67 test cases mapped to success criteria
- **Per-Crate APIs**: `crates/*.md` for implementation-ready specifications

---

**Design Version**: 1.0  
**Status**: ✅ COMPLETE (Ready for PROVE phase)  
**Authors**: Design generated via rust-whiteboard-design-docs skill  
**Review Date**: 2026-03-07
