# STRATEGY

## Overview

This document defines the high-level strategy for adding persistent storage to the Whirlpool mempool. The goal is to preserve pending transactions across node restarts while maintaining existing consensus semantics and minimizing invasiveness.

**Current State [GROUNDED]**: `InMemoryTxPool` stores transactions in `Mutex<Vec<Vec<u8>>>` — txs are lost on restart. `TxSource` trait has only `pending()` method. `EvmApplication` already uses trait objects (`Arc<dyn TxSource>`), but `EthRpcContext` uses concrete `Arc<InMemoryTxPool>`.

**Target State [PROPOSED]**: `PersistentTxPool` stores transactions in embedded database, implements extended `TxSource` trait with `push()` method, plugs into existing wiring with zero consensus changes.

---

## Core Design Decisions

### 1. Persistence Backend: Raw libmdbx [PROPOSED]

**Decision**: Use raw `libmdbx-rs` crate directly (NOT reth-db).

**Rationale**:
- **[GROUNDED]** reth-db's `Table` trait and `Tables` enum are tightly coupled. Extending `Tables` requires vendor modification (forbidden).
- **[GROUNDED]** state-reth already uses reth-db for state persistence, but mempool needs independent tables.
- **[GROUNDED]** MDBX databases coexist independently via directory paths — each `open_db(path)` yields separate `DatabaseEnv`.
- **[PROPOSED]** Mempool storage is simple (key → raw bytes) — full reth-db abstractions are overkill.
- **[PROPOSED]** `libmdbx-rs` provides direct MDBX bindings with low overhead, proven reliability, and concurrent read + single-writer guarantees.

**Alternative Considered: redb**
- Pure Rust (safety), simpler API than raw MDBX.
- **Rejected**: MDBX is already in-tree via reth-db, battle-tested at scale. Adding another DB dependency for this use case introduces unnecessary divergence.

**Alternative Considered: sled**
- Embedded KV store, async-friendly.
- **Rejected**: sled is beta, less mature than MDBX for production workloads.

**Alternative Considered: File-based (JSON/bincode)**
- Simplest possible persistence.
- **Rejected**: No transactional semantics, poor concurrency, slow on large volumes.

### 2. TxSource Trait Extension: Add `push()` Method [PROPOSED]

**Decision**: Extend `TxSource` trait to include `fn push(&self, tx: Vec<u8>)`.

**Rationale**:
- **[GROUNDED]** Current trait: `pub trait TxSource { fn pending(&self) -> Vec<Vec<u8>>; }` (app/src/traits.rs:23).
- **[GROUNDED]** RPC layer needs `push()` to submit txs: `ctx.tx_pool.push(bytes.to_vec())` (rpc-eth/src/context.rs).
- **[GROUNDED]** `EthRpcContext` currently holds `Arc<InMemoryTxPool>` concrete type to access `push()`.
- **[PROPOSED]** Adding `push()` to trait enables `EthRpcContext` to hold `Arc<dyn TxSource>` and use trait object.
- **[GROUNDED]** Only 3 implementors to update: `InMemoryTxPool`, `NoopTxSource`, test mocks.

**Signature [PROPOSED]**:
```rust
pub trait TxSource: Send + Sync {
    fn pending(&self) -> Vec<Vec<u8>>;
    fn push(&self, tx: Vec<u8>);  // New
}
```

**Impact [GROUNDED]**:
- `app/src/tx_source.rs`: Update `InMemoryTxPool` impl (already has `push`), add `push` to `NoopTxSource` (no-op).
- `app-evm/tests/integration.rs`: Update `MockTxSource` impl.
- `rpc-eth/src/eth_handler.rs`: Update test helpers `mock_tx_pool()`.

### 3. Storage Key Strategy: Auto-Increment u64 [PROPOSED]

**Decision**: Use auto-incrementing u64 as primary key, store raw EIP-2718 bytes as value.

**Schema [PROPOSED]**:
```
Table: pending_txs
Key: u64 (insertion order counter)
Value: Vec<u8> (raw EIP-2718 encoded transaction bytes)
```

**Rationale**:
- **[GROUNDED]** InMemoryTxPool stores `Vec<Vec<u8>>` in FIFO order — drain returns oldest first.
- **[GROUNDED]** Consensus layer expects FIFO ordering from `pending()` (app-evm/src/executor.rs:155).
- **[PROPOSED]** u64 key preserves insertion order naturally — scan ascending on `pending()`.
- **[PROPOSED]** Cheap insert: no decoding, no hashing — just `next_id.fetch_add(1)` + `put(id, bytes)`.
- **[GROUNDED]** `TransactionSigned::decode_2718` and hashing are deferred to `EvmApplication.propose()` (already happens there).

**Alternative Considered: Tx Hash as Key**
- **Benefit**: Natural deduplication (same tx submitted twice overwrites).
- **Cost**: Must decode + hash on every `push()` (keccak256 overhead).
- **[GROUNDED]** InMemoryTxPool does NOT deduplicate today — stores duplicates.
- **Rejected**: Adding dedup changes semantics beyond persistence goal. Can add hash index later as optimization.

**Metadata [PROPOSED]**:
- Store global counter in separate key: `Key::Counter → u64`
- On startup: load max key from `pending_txs` table, resume counter.

### 4. Drain Semantics: Delete on `pending()` [PROPOSED]

**Decision**: `pending()` drains DB (deletes returned txs), matching InMemoryTxPool behavior.

**Rationale**:
- **[GROUNDED]** `InMemoryTxPool.pending()` drains Vec via `std::mem::take(&mut *inner)` (app/src/tx_source.rs:36).
- **[GROUNDED]** Post-drain, txs exist nowhere — if consensus fails to finalize, they are lost (current behavior).
- **[PROPOSED]** Match this behavior initially: `pending()` reads all txs (ascending order), deletes them from DB, returns Vec.
- **[PROPOSED]** Use MDBX transaction to atomically: (1) read all, (2) delete all, (3) commit.

**Crash Recovery [PROPOSED]**:
- On startup: all txs in DB are unproposed → immediately available to next `pending()` call.
- Crash between propose and finalize: txs lost (same as today, acceptable for MVP).

**Future Enhancement [PROPOSED]**:
- Track lifecycle: `submitted` → `proposed` → `finalized`. On crash, re-queue `proposed` txs.
- Requires: additional `proposed_txs` table, finalization callback integration.
- **Out of scope** for initial implementation — preserve current semantics first.

### 5. EthRpcContext Generification: Use Trait Object [PROPOSED]

**Decision**: Change `EthRpcContext.tx_pool` from `Arc<InMemoryTxPool>` to `Arc<dyn TxSource + Send + Sync>`.

**Rationale**:
- **[GROUNDED]** `EvmApplication` already uses `Arc<dyn TxSource + Send + Sync>` (app-evm/src/executor.rs:48).
- **[GROUNDED]** `EthRpcContext` currently: `tx_pool: Arc<InMemoryTxPool>` (rpc-eth/src/context.rs:14).
- **[GROUNDED]** RPC only calls `push(tx)` — once trait has `push()`, no concrete type needed.
- **[PROPOSED]** Trait object avoids type parameter propagation through `EthApiHandler`, `EthApiServer`, `start_rpc_server`.
- **[PROPOSED]** Minimal diff: change field type, update `new()` signature, no logic changes.

**Implementation [PROPOSED]**:
```rust
pub struct EthRpcContext<S: StateDb, B: BlockStorage> {
    tx_pool: Arc<dyn TxSource + Send + Sync>,  // Changed
    state_db: Arc<RwLock<S>>,
    block_storage: Arc<B>,
    receipt_store: ReceiptStore,
    chain_id: u64,
    block_height: AtomicU64,
}
```

**Clone impl**: Already clones Arcs — no change needed (trait object is Clone via Arc).

---

## Crate Organization

### Option A: New Crate `mempool` [PROPOSED — Preferred]

**Structure**:
```
crates/mempool/
  src/
    lib.rs         — re-exports
    persistent.rs  — PersistentTxPool impl
    store.rs       — raw libmdbx wrapper
  Cargo.toml       — depends on libmdbx-rs, app
```

**Rationale**:
- **[PROPOSED]** Clear responsibility: mempool persistence is orthogonal to app-layer traits.
- **[PROPOSED]** Encapsulates MDBX dependency — `app` stays DB-agnostic.
- **[PROPOSED]** Parallel to `state-reth` (state storage) and `rpc-eth` (RPC handling).
- **[PROPOSED]** Tests isolated in `mempool/tests/` — can test persistence independently.

**Dependencies**:
- `libmdbx-rs`: Direct MDBX bindings
- `app`: For `TxSource` trait
- `parking_lot`: For `Mutex` (lighter than std::sync::Mutex)

### Option B: Extend `app` Crate [PROPOSED — Alternative]

**Structure**:
```
crates/app/
  src/
    lib.rs
    traits.rs      — TxSource trait
    tx_source.rs   — InMemoryTxPool, NoopTxSource, PersistentTxPool
  Cargo.toml       — add libmdbx-rs optional dep
```

**Rationale**:
- **[GROUNDED]** `app` already has `InMemoryTxPool`, natural to add persistent variant.
- **[PROPOSED]** Fewer crates, simpler workspace.
- **[PROPOSED]** MDBX dep behind feature flag: `persistence = ["libmdbx-rs"]`.

**Trade-off**:
- Mixes trait definitions with concrete storage implementations.
- `app` becomes heavier (includes DB code).

**Recommendation**: **Option A** — new crate for cleaner separation.

---

## Implementation Ordering

### Phase 1: Trait Foundation [MUST BE FIRST]
1. **Extend `TxSource` trait** (app/src/traits.rs)
   - Add `fn push(&self, tx: Vec<u8>)` to trait.
   - **Dependency**: None — foundational change.
   - **Risk**: Breaks compilation of all implementors until next step.

2. **Update existing implementors** (app/src/tx_source.rs, tests)
   - `InMemoryTxPool::push`: Already exists, no-op (already impl outside trait).
   - `NoopTxSource::push`: Add empty impl.
   - Test mocks: Add empty impl.
   - **Dependency**: Step 1 complete.

3. **Verify app crate builds**
   - `nix develop --command cargo build -p app`
   - **Gate**: All implementors compile, tests pass.

### Phase 2: RPC Generification [DEPENDS ON PHASE 1]
4. **Generify `EthRpcContext`** (rpc-eth/src/context.rs)
   - Change `tx_pool` field type to `Arc<dyn TxSource + Send + Sync>`.
   - Update `new()` signature to accept trait object.
   - **Dependency**: Step 2 complete (trait has `push()`).

5. **Update node wiring** (whirlpool-node/src/main.rs)
   - Wrap `InMemoryTxPool` in Arc, cast to trait object if needed (should be automatic via coercion).
   - **Dependency**: Step 4 complete.

6. **Verify rpc-eth + node build**
   - `nix develop --command cargo build -p rpc-eth -p whirlpool-node`
   - **Gate**: Compilation succeeds, no behavior change (still using InMemoryTxPool).

### Phase 3: Persistent Implementation [PARALLEL TO PHASE 2 AFTER PHASE 1]
7. **Create mempool crate skeleton**
   - `Cargo.toml`, `lib.rs`, `persistent.rs`, `store.rs`
   - Add `libmdbx-rs` dependency.
   - **Dependency**: None (independent of RPC changes).

8. **Implement MDBX wrapper** (mempool/src/store.rs)
   - `MempoolStore::open(path)` → Environment + DB handle
   - `push(tx: Vec<u8>)` → auto-increment + insert
   - `drain_pending()` → read all + delete + commit
   - **Dependency**: Step 7 complete.

9. **Implement `TxSource` for `PersistentTxPool`** (mempool/src/persistent.rs)
   - Wrap `MempoolStore` in `Arc<Mutex<MempoolStore>>` for interior mutability.
   - `pending()` delegates to `store.drain_pending()`.
   - `push()` delegates to `store.push()`.
   - **Dependency**: Steps 2, 8 complete.

10. **Write unit tests** (mempool/tests/persistence.rs)
    - Test: push → pending drains → empty
    - Test: push → restart → pending recovers txs
    - Test: concurrent push (stress test)
    - **Dependency**: Step 9 complete.

11. **Verify mempool crate builds**
    - `nix develop --command cargo build -p mempool`
    - `nix develop --command cargo test -p mempool`
    - **Gate**: Tests pass, persistence verified.

### Phase 4: Integration [DEPENDS ON PHASES 2 & 3]
12. **Wire `PersistentTxPool` in node** (whirlpool-node/src/main.rs)
    - Add `mempool` dependency to whirlpool-node Cargo.toml.
    - Compute persistent path: `{persistent_storage_dir}/mempool`.
    - Replace `InMemoryTxPool::new()` with `PersistentTxPool::open(path)?`.
    - Wrap in Arc, pass to `EvmApplication::new` and `EthRpcContext::new`.
    - **Dependency**: Steps 6, 11 complete.

13. **Integration test** (app-evm/tests/integration.rs or new test in node)
    - Submit tx via RPC → verify pending() returns it.
    - Drain via propose → verify DB empty.
    - Restart node → verify recovery.
    - **Dependency**: Step 12 complete.

14. **Full workspace build + test**
    - `nix develop --command cargo build`
    - `nix develop --command cargo test`
    - **Gate**: All tests pass, no regressions.

### Phase 5: Cleanup [OPTIONAL]
15. **Benchmarks** (if needed)
    - Compare InMemoryTxPool vs PersistentTxPool latency on push/pending.
    - Document overhead (expect <1ms per op for reasonable volumes).

16. **Documentation**
    - Update crate README.md files.
    - Document storage path management.
    - Document recovery semantics.

### Dependency Graph
```
1 (trait) → 2 (impls) → 3 (verify app)
                      ↘
                        4 (rpc) → 5 (node wiring) → 6 (verify)
                                                           ↘
1 → 7 (crate) → 8 (store) → 9 (persistent) → 10 (tests) → 11 (verify) → 12 (integrate) → 13 (test) → 14 (full)
```

**Critical Path**: Steps 1-6 (trait + RPC) must complete before step 12 (integration). Step 7-11 (persistence impl) can proceed in parallel after step 1.

---

## Migration Strategy

### Backward Compatibility [GROUNDED + PROPOSED]

**Trait Extension [PROPOSED]**:
- Adding `push()` to `TxSource` is breaking for out-of-tree implementors (none exist [GROUNDED]).
- In-tree implementors (InMemoryTxPool, NoopTxSource) updated in same commit.

**Runtime [PROPOSED]**:
- `InMemoryTxPool` remains default during development (Phase 1-2).
- `PersistentTxPool` introduced as opt-in (Phase 3).
- Final integration (Phase 4) switches default to persistent.

### Deployment [PROPOSED]

**Initial Deployment**:
- Ship with `PersistentTxPool` as default.
- Fresh nodes: empty DB, normal operation.
- Existing nodes: old `InMemoryTxPool` state lost on upgrade (acceptable — mempool is transient).

**Rollback [PROPOSED]**:
- If critical issue discovered: revert node wiring to `InMemoryTxPool::new()`, remove mempool crate dependency.
- No state migration needed (mempool txs are transient).

### Data Migration [PROPOSED]

**Not Applicable**: InMemoryTxPool state is in-memory only — no data to migrate on upgrade.

**Schema Evolution [PROPOSED]**:
- If future schema changes needed: version metadata in DB (`schema_version` key).
- On open: check version, migrate or error.
- Initial version: 1.

---

## Technology Justification

### libmdbx [PROPOSED]
- **Maturity**: LMDB fork with 10+ years of production use, used by reth via reth-db.
- **Performance**: ACID transactions, mmap-based, zero-copy reads, concurrent readers + single writer.
- **Rust Bindings**: `libmdbx-rs` crate provides safe wrappers, well-maintained.
- **Footprint**: Embedded, no separate process, minimal dependencies.

### Alternatives Rejected

**redb [PROPOSED]**:
- Pure Rust, simpler API, type-safe tables.
- **Rejected**: Less mature than MDBX, different dependency from state-reth (divergence), unproven at Whirlpool's scale.

**sled [PROPOSED]**:
- Embedded, async-friendly, pure Rust.
- **Rejected**: Beta quality, known correctness issues in past, less adoption than MDBX.

**PostgreSQL/SQLite [PROPOSED]**:
- Full RDBMS features, SQL interface.
- **Rejected**: Overkill for KV use case, external process (PostgreSQL) or C dependency (SQLite), higher complexity.

**Custom File Format [PROPOSED]**:
- Simple append-log or indexed file.
- **Rejected**: No transactional guarantees, complex concurrency handling, reinventing the wheel.

---

## Risks & Mitigations

### Risk 1: reth-db Table Coupling [GROUNDED]
**Description**: reth-db's `Tables` enum cannot be extended without vendor changes.

**Impact**: Cannot use reth-db for mempool tables.

**Mitigation [PROPOSED]**: Use raw `libmdbx-rs` — separate DB directory, zero vendor coupling.

**Status**: Mitigated by design decision (Section 1).

### Risk 2: TxSource Trait Breaking Change [GROUNDED]
**Description**: Adding `push()` breaks existing implementors.

**Impact**: Compilation failure until all impls updated.

**Mitigation [PROPOSED]**:
- All implementors in-tree [GROUNDED] — updated in same commit (Phase 1).
- No external crates depend on Whirlpool `TxSource` trait [GROUNDED].

**Status**: Low risk, controlled change.

### Risk 3: Crash Between Propose and Finalize [GROUNDED]
**Description**: `pending()` drains DB; if node crashes before finalization, txs lost.

**Impact**: Same as today with InMemoryTxPool — acceptable for MVP.

**Mitigation [PROPOSED]**:
- Document behavior: mempool is not durable across propose-finalize gap.
- Future: implement `proposed` lifecycle state, re-queue on restart.

**Status**: Accepted risk for initial implementation.

### Risk 4: Persistence Performance Overhead [PROPOSED]
**Description**: DB write on every `push()` could slow RPC submission.

**Impact**: Higher p99 latency for `eth_sendRawTransaction`.

**Mitigation [PROPOSED]**:
- MDBX is fast (mmap-based, write-optimized).
- Mempool ops are NOT hot path [GROUNDED] — consensus/execution dominates latency.
- Benchmark: expect <1ms overhead per tx on modern hardware.

**Status**: Low likelihood; measurable in Phase 5 if needed.

### Risk 5: Deduplication Not Addressed [GROUNDED]
**Description**: InMemoryTxPool stores duplicates; PersistentTxPool will too (with auto-increment keys).

**Impact**: Duplicate txs waste disk space, included multiple times in proposals.

**Mitigation [PROPOSED]**:
- Not a regression — current behavior preserved.
- Future: add hash index for dedup (Phase 5 enhancement).

**Status**: Out of scope for MVP.

### Risk 6: Storage Path Misconfiguration [PROPOSED]
**Description**: If mempool path collides with state DB path, corruption possible.

**Impact**: Data loss, undefined behavior.

**Mitigation [PROPOSED]**:
- Use dedicated subdir: `{persistent_storage_dir}/mempool`.
- Document path structure clearly.
- Validate path doesn't overlap state-reth or block storage.

**Status**: Mitigated by implementation (Phase 4, Step 12).

---

## Rejected Alternatives

### Alternative 1: Extend reth-db Tables Enum
**Description**: Modify vendor reth-db to add `MempoolTxs` table.

**Rejected**: Violates no-vendor-modification constraint [GROUNDED]. Creates maintenance burden, divergence from upstream.

### Alternative 2: Generic Type Parameter for EthRpcContext
**Description**: `EthRpcContext<S, B, T: TxSource>` instead of trait object.

**Rejected**:
- Propagates type parameter through `EthApiHandler`, `EthApiServer`, `start_rpc_server` [GROUNDED].
- More invasive than trait object approach [PROPOSED].
- No benefit — RPC doesn't need compile-time monomorphization for tx pool.

### Alternative 3: Separate Push/Drain Traits
**Description**: `TxSink { fn push(...) }` + `TxSource { fn pending(...) }`.

**Rejected**:
- Splits concerns unnecessarily — mempool needs both push and drain.
- Complicates wiring — node must manage two trait objects.
- RPC needs `push`, consensus needs `pending` — but implementor needs both.

### Alternative 4: Async TxSource Trait
**Description**: `async fn pending(&self) -> Vec<Vec<u8>>`.

**Rejected**:
- [GROUNDED] Current trait is sync, all callers sync (EvmApplication.propose() is sync).
- MDBX is sync-only — no benefit to async trait.
- Would require async/sync boundary adapters everywhere.

### Alternative 5: JSON File Persistence
**Description**: Serialize pending txs to JSON file on each `push()`.

**Rejected**:
- No transactional semantics — crash mid-write corrupts file.
- Poor concurrency — must lock entire file for read/write.
- Slow for large mempool (full rewrite on each push).

---

## Open Questions

None. All design decisions resolved based on exploration findings.

---

## Success Criteria

1. **Functional**: Transactions submitted via RPC survive node restart, returned by next `pending()` call.
2. **Semantic Preservation**: Drain behavior identical to InMemoryTxPool (FIFO order, no re-queue).
3. **Integration**: Zero changes to consensus layer (EvmApplication) or finalization logic.
4. **Correctness**: `cargo test` passes for all crates after each phase.
5. **Build**: `nix develop --command cargo build` succeeds for full workspace.
6. **Documentation**: Each crate's public API documented with rustdoc.

---

## Summary

**High-Level Approach [PROPOSED]**: Add new `mempool` crate with `PersistentTxPool` implementing extended `TxSource` trait. Use raw `libmdbx-rs` for embedded storage, auto-increment keys for FIFO ordering. Generify `EthRpcContext` to trait object, wire persistent pool in `whirlpool-node` main.

**Key Decisions**:
1. Raw libmdbx (not reth-db) — avoids vendor coupling.
2. Extend `TxSource` trait with `push()` — enables trait object use.
3. Auto-increment u64 keys — preserves FIFO, cheap inserts.
4. Drain-on-pending semantics — matches current behavior.
5. Trait object generification — minimal propagation, matches EvmApplication pattern.

**Implementation Phases**: Trait foundation → RPC generification → Persistent impl → Integration. Phases 2-3 parallelizable after Phase 1. Critical path: 14 steps, ~3-5 days estimated effort.

**Risk Profile**: Low — all risks mitigated by design or accepted as MVP constraints. No vendor changes, no consensus changes, backward-compatible trait extension, independent DB directory.
