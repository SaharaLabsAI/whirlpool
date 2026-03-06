# Proof — Persistent Block Storage & History Queries

**Sub-intent slug**: persistent-blocks-history
**Source design docs**: `../docs/`

---

## S0 — Pre-conditions

### Blocker Check
- **Active blockers**: 0 (BLOCKERS.md)
- **Deferred blockers**: 8 (BLK-1..7, BLK-11) — all have mitigations noted, none block MVP implementation
- **Resolved**: 3 (BLK-8..10)
- **Verdict**: No blockers prevent proceeding to implementation

### Design Completeness
- INTENT.md: 5 success criteria (SC-1..SC-5) defined ✅
- STRATEGY.md: 3 streams, 5 key decisions ✅
- CRATES.md: 7 crates with change descriptions ✅
- DOMAINS.md: 5 domains + unaffected crates noted ✅
- FLOWS.md: 4 architecture flows with [EXISTING]/[NEW] annotations ✅
- TESTS.md: 22 unit + 2 integration + 4 flow tests ✅
- Per-crate contracts: 7 files (state, state-reth, app, app-evm, rpc-eth, whirlpool-node, consensus-simplex) ✅
- INDEX.md + SUMMARY.md: Generated ✅
- Oracle self-check: PASS (6 issues found and fixed) ✅

### Evidence Traceability
Each success criterion traces through the full design chain:

| SC | STRATEGY Stream | DOMAINS Domain | Crate Contract | Flow | Tests |
|----|----------------|----------------|---------------|------|-------|
| SC-1 | Stream 1 (BlockStorage) | Storage | state.md, state-reth.md | Flow 1 steps 7-10 | TC-SR-01, TC-SR-07 |
| SC-2 | Stream 2 (Finalization) | App + Wiring | app-evm.md, whirlpool-node.md | Flow 1 steps 4-6 | TC-INT-01 |
| SC-3 | Stream 3 (RPC) | RPC/Query | rpc-eth.md | Flow 2 | TC-RPC-02, TC-RPC-05, TC-RPC-07 |
| SC-4 | Stream 3 (RPC) | RPC/Query | rpc-eth.md | Flow 3 | TC-RPC-04 |
| SC-5 | All 3 streams | Node Wiring | whirlpool-node.md | Flow 4 | TC-INT-02 |

---

## S1 — Split Justification

**Decision**: No split required — single intent.

**Rationale**: This feature is a single cohesive unit with tight coupling between its three components:
1. Storage (BlockStorage trait + MDBX impl) is exclusively consumed by finalization and RPC
2. Finalization persistence is meaningless without storage
3. RPC history queries are meaningless without persisted data

Splitting would create partial implementations that cannot be independently tested end-to-end. The feature boundary is already well-scoped (5 success criteria, 7 crates, 4 flows) and achievable in a single implementation pass.

**Sub-intent slug**: `persistent-blocks-history` (the entire feature)

---

## S2 — Invariants

### INV-1: MDBX Atomic Multi-Table Commit
- **Scope**: `state-reth`
- **Statement**: `store_block` writes `Headers`, `CanonicalHeaders`, `HeaderNumbers`, `BlockBodyIndices`, `Transactions`, `TransactionHashNumbers`, `TransactionBlocks`, and `Receipts` in one MDBX write transaction; if any write fails, none are committed.
- **Grounding**: `INTENT.md` SC-1; `STRATEGY.md` Stream 1 (single write transaction); `FLOWS.md` Flow 1 step 10.
- **Verification**: `TC-SR-01`, `TC-SR-02`, `TC-FLW-01`.

### INV-2: Global TxNumber Monotonicity
- **Scope**: `state-reth`
- **Statement**: `TxNumber` assignment is append-only and strictly increasing across blocks, with each new block starting at prior `first_tx_num + tx_count`.
- **Grounding**: `STRATEGY.md` Key Decision 5; `crates/state-reth.md` Transaction Numbering Strategy; `FLOWS.md` Flow 1 step 9.
- **Verification**: `TC-SR-07`.

### INV-3: Block Reconstruction Fidelity
- **Scope**: `state-reth`, `app-evm`
- **Statement**: For a persisted block, `get_block_by_number(block.height)` reconstructs an equivalent `EvmBlock` (header fields and ordered transaction payloads preserved through encode/decode).
- **Grounding**: `STRATEGY.md` Stream 1 read strategy; `crates/state-reth.md` EvmBlock reconstruction contract; `FLOWS.md` Flow 2 step 4.
- **Verification**: `TC-SR-03`, `TC-INT-01`, `TC-UNK-02`.

### INV-4: Hash to Number Consistency
- **Scope**: `state-reth`, `rpc-eth`
- **Statement**: Hash lookup is consistent with number lookup: `get_block_by_hash(h)` returns the same block as `get_block_by_number(n)` where `HeaderNumbers[h] = n`.
- **Grounding**: `STRATEGY.md` Stream 1 table usage; `FLOWS.md` Flow 3 steps 3-4; `crates/state-reth.md` `get_block_by_hash` design.
- **Verification**: `TC-SR-05`, `TC-RPC-04`, `TC-FLW-03`.

### INV-5: Receipt Count Equals Transaction Count
- **Scope**: `app-evm`, `state-reth`
- **Statement**: Receipts persisted per block match that block's transaction count; mismatches are rejected and transaction is aborted.
- **Grounding**: `INTENT.md` SC-1; `FLOWS.md` Flow 1 step 6 (mismatch edge) and step 10 (atomic abort); `crates/state-reth.md` write contract.
- **Verification**: `TC-SR-01`, `TC-SR-02`, `TC-SR-08`.

### INV-6: Finalization Idempotency Safety
- **Scope**: `app-evm`, `state-reth`, `whirlpool-node`
- **Statement**: Re-processing finalization for the same block does not corrupt persisted mappings; duplicate handling is no-op or equivalent-state safe.
- **Grounding**: `STRATEGY.md` Stream 2 receipt lifecycle; `FLOWS.md` Flow 1 step 5 duplicate-finalization edge; `crates/app-evm.md` `pending_receipts.take().unwrap_or_default()` lifecycle.
- **Verification**: `TC-AE-02`, `TC-AE-03`, `TC-UNK-01`.

### INV-7: Consensus Independence from Storage Failure
- **Scope**: `app-evm`, `whirlpool-node`, `consensus-simplex`
- **Statement**: Storage failures during finalization are logged and do not halt consensus/finalization progression or finalized height updates.
- **Grounding**: `INTENT.md` constraints (must not break consensus); `STRATEGY.md` persistence at app layer (consensus-simplex unchanged); `crates/whirlpool-node.md` non-fatal error policy; `FLOWS.md` Flow 1 step 4.
- **Verification**: `TC-AE-04`, `TC-CS-01`, `TC-UNK-03`.

### INV-8: RPC Type Fidelity for Block Responses
- **Scope**: `rpc-eth`
- **Statement**: `evm_block_to_rpc_block` yields valid JSON-RPC block responses with correct field mapping and mode behavior (`full=true` full tx objects, `full=false` tx hashes), with decode failures surfaced as RPC internal errors.
- **Grounding**: `INTENT.md` SC-3 and SC-4; `STRATEGY.md` Stream 3 conversion requirements; `crates/rpc-eth.md` conversion and error mapping; `FLOWS.md` Flow 2 step 5.
- **Verification**: `TC-RPC-02`, `TC-RPC-03`, `TC-RPC-04`, `TC-RPC-08`.

### INV-9: Existing Functionality Preservation
- **Scope**: workspace-wide (`state`, `consensus`, `consensus-simplex`, `rpc-eth`, `whirlpool-node`)
- **Statement**: Existing state operations and pre-existing consensus/RPC behavior remain unchanged; block history support is additive.
- **Grounding**: `INTENT.md` SC-5 and constraints; `WORKSPACE.md` backward compatibility constraints; `DOMAINS.md` unchanged boundaries and unaffected crates.
- **Verification**: `TC-CS-01`, `TC-INT-02`, workspace regression checks in `WORKSPACE.md`.

### INV-10: Thread Safe Shared BlockStorage Access
- **Scope**: `state`, `rpc-eth`, `whirlpool-node`, `app-evm`
- **Statement**: Finalization writes and RPC reads share storage through thread-safe ownership (`BlockStorage: Send + Sync` and `Arc<RwLock<...>>`).
- **Grounding**: `crates/state.md` trait bounds; `WORKSPACE.md` Integration Point 4 concurrency; `crates/whirlpool-node.md` shared `Arc<RwLock<RethStateDb>>` wiring.
- **Verification**: `TC-ST-01`, `TC-INT-02`, `TC-FLW-04`.

---

## S3 — Acceptance Criteria

### AC-1: Header and Canonical Indexes Persist Atomically
- **SC**: SC-1
- **Criterion**: On finalized block persistence, header-oriented records (`Headers`, `CanonicalHeaders`, `HeaderNumbers`) are committed in the same MDBX transaction as body/tx/receipt records, never independently.
- **QA Scenario**: QA-1: (1) Finalize a block with known header fields; (2) inspect DB in one test transaction; (3) verify all header mappings exist and are coherent with block number/hash; (4) inject write failure and assert no partial commit.
- **Invariants**: INV-1, INV-4

### AC-2: Body and Transaction Indexes Persist Atomically
- **SC**: SC-1
- **Criterion**: Persisting a finalized block writes `BlockBodyIndices`, `Transactions`, `TransactionHashNumbers`, and `TransactionBlocks` in one atomic commit with contiguous tx numbering.
- **QA Scenario**: QA-2: (1) Store blocks with varying tx counts; (2) verify `first_tx_num` and `tx_count` ranges and contiguous `TxNumber` assignments; (3) confirm reverse tx mappings resolve to the block.
- **Invariants**: INV-1, INV-2

### AC-3: Receipts Persist with Matching Cardinality
- **SC**: SC-1
- **Criterion**: Receipts for a block are persisted in the same commit and cardinality-equivalent to decoded block transactions; mismatch causes atomic failure with no partial writes.
- **QA Scenario**: QA-3: (1) Store block with N tx and N receipts; (2) query receipts by block and verify count/order; (3) retry with mismatched receipts and assert operation fails and tables remain unchanged.
- **Invariants**: INV-1, INV-5

### AC-4: Finalization Automatically Triggers Persistence
- **SC**: SC-2
- **Criterion**: Receiving `ConsensusEvent::Finalized` triggers block persistence without external/manual invocation, through app and node finalization wiring.
- **QA Scenario**: QA-4: (1) Propose and finalize a block via consensus flow; (2) do not call storage APIs directly; (3) verify block is retrievable from MDBX by number.
- **Invariants**: INV-3, INV-7

### AC-5: Propose-Time Receipts Are Captured and Flushed on Finalization
- **SC**: SC-2
- **Criterion**: Receipts produced during `propose()` are cached (`pending_receipts`) and consumed by finalization persistence; cache is cleared after store to avoid stale reuse.
- **QA Scenario**: QA-5: (1) Run `propose()` and assert receipts cached; (2) finalize block and assert `store_block` called with those receipts; (3) assert cache becomes `None`; (4) duplicate finalize does not corrupt state.
- **Invariants**: INV-5, INV-6

### AC-6: getBlockByNumber Resolves Numeric Heights
- **SC**: SC-3
- **Criterion**: `eth_getBlockByNumber(Number(n), full)` returns persisted block `n` when present and `null` when absent, using the storage reconstruction path.
- **QA Scenario**: QA-6: (1) Persist block at known height; (2) call RPC with exact numeric height for `full=true` and `full=false`; (3) verify successful response; (4) query missing height and verify `null`.
- **Invariants**: INV-3, INV-8

### AC-7: getBlockByNumber Resolves Tags per MVP Policy
- **SC**: SC-3
- **Criterion**: Tag resolution follows design policy: `Latest`, `Finalized`, and `Safe` map to finalized height, `Earliest` maps to block 0, and `Pending` returns `null` for MVP.
- **QA Scenario**: QA-7: (1) Set finalized height in context; (2) call RPC with `latest`, `finalized`, `safe`, `earliest`, and `pending`; (3) verify mapped storage calls and response semantics.
- **Invariants**: INV-8, INV-9

### AC-8: getBlockByNumber Honors Full-vs-Hash Transaction Modes
- **SC**: SC-3
- **Criterion**: For the same persisted block, `full=true` returns full transaction objects while `full=false` returns only transaction hashes, with valid JSON-RPC shape.
- **QA Scenario**: QA-8: (1) Persist a block with multiple txs; (2) call endpoint twice with `full=true` and `full=false`; (3) compare transaction payload structure and hashes.
- **Invariants**: INV-8

### AC-9: getBlockByHash Returns Persisted Block via HeaderNumbers Lookup
- **SC**: SC-4
- **Criterion**: `eth_getBlockByHash(hash, full)` resolves hash through `HeaderNumbers` and returns the same canonical block as number-based lookup; unknown hash returns `null`.
- **QA Scenario**: QA-9: (1) Persist block and compute hash; (2) query by hash and by number; (3) assert equivalent block identity; (4) query random hash and assert `null`.
- **Invariants**: INV-4, INV-8

### AC-10: getBlockByHash Honors Full-vs-Hash Transaction Modes
- **SC**: SC-4
- **Criterion**: Hash-based endpoint supports both transaction rendering modes exactly as number-based endpoint (`full=true` objects, `full=false` hashes).
- **QA Scenario**: QA-10: (1) Persist block; (2) call `eth_getBlockByHash` with both `full` values; (3) validate transaction representation mode and field fidelity.
- **Invariants**: INV-8

### AC-11: Node Startup Wires Shared Persistent Storage into App and RPC
- **SC**: SC-5
- **Criterion**: Node boot initializes one `RethStateDb` and wires it into both EVM app (write path) and RPC context (query path), enabling end-to-end persistence/query on a single shared backend.
- **QA Scenario**: QA-11: (1) Start node; (2) finalize at least one block; (3) call `eth_getBlockByNumber("latest", true)`; (4) verify data is served from persistent storage and startup wiring compiles/runs without type mismatch.
- **Invariants**: INV-10, INV-3

### AC-12: Existing Consensus and RPC Flows Remain Unbroken
- **SC**: SC-5
- **Criterion**: Integrating persistence/query does not regress existing consensus progression, height tracking, or pre-existing RPC/state operations; storage errors are non-fatal to consensus.
- **QA Scenario**: QA-12: (1) Execute existing consensus and RPC smoke flow; (2) inject or observe storage error path during finalization; (3) verify node continues finalization and existing methods still succeed.
- **Invariants**: INV-7, INV-9

### Coverage Matrix

| AC | QA | INV | SC |
|----|----|----|-----|
| AC-1 | QA-1 | INV-1, INV-4 | SC-1 |
| AC-2 | QA-2 | INV-1, INV-2 | SC-1 |
| AC-3 | QA-3 | INV-1, INV-5 | SC-1 |
| AC-4 | QA-4 | INV-3, INV-7 | SC-2 |
| AC-5 | QA-5 | INV-5, INV-6 | SC-2 |
| AC-6 | QA-6 | INV-3, INV-8 | SC-3 |
| AC-7 | QA-7 | INV-8, INV-9 | SC-3 |
| AC-8 | QA-8 | INV-8 | SC-3 |
| AC-9 | QA-9 | INV-4, INV-8 | SC-4 |
| AC-10 | QA-10 | INV-8 | SC-4 |
| AC-11 | QA-11 | INV-10, INV-3 | SC-5 |
| AC-12 | QA-12 | INV-7, INV-9 | SC-5 |

---

## S4 — Dependency Contract

### New Internal Dependencies

| From | To | Reason | Breaking? |
|------|----|--------|-----------|
| `state` | `app` | `EvmBlock` type in `BlockStorage` trait signature | No — additive trait |
| `state` | `alloy-consensus` | `Receipt` type in trait signature | No — additive |
| `state-reth` | `app-evm` | `build_header_from_evm_block()`, `decode_transactions()` | No — uses existing functions (visibility change only) |
| `rpc-eth` | `app-evm` | `evm_block_to_rpc_block` conversion helpers, EvmBlock→Block mapping | No — additive endpoint |
| `rpc-eth` | `state` | `BlockStorage` trait (already depends on `state` for `StateDb`) | No — trait addition |
| `app` | `alloy-consensus` | `Receipt` re-export | No — additive re-export |

### New External Dependencies

| Crate | External Dep | Version | Purpose |
|-------|-------------|---------|---------|
| `state` | `alloy-consensus` | `1.4.3` | Receipt type for BlockStorage trait |
| `app` | `alloy-consensus` | `1.4.3` | Receipt re-export |

Note: `state-reth` already depends on `reth-db`, `alloy-primitives`, and other reth/alloy crates. Any transitive needs for `alloy-eips` or `reth-ethereum-primitives` are already satisfied through existing dependency chains.

### Feature Flags
No new feature flags introduced. All changes are unconditional.

### Breaking Changes
**None.** All changes are additive:
- `BlockStorage` is a new trait (no existing trait modified)
- `EthRpcContext` gains a new generic parameter `B: BlockStorage` — this is a breaking change to `EthRpcContext`'s type signature, but it's internal to the workspace (not a published API). All call sites in `whirlpool-node` are updated together.
- `build_header_from_evm_block` visibility changes from `pub(crate)` to `pub` — strictly widening, not breaking.
- Two new RPC methods added to `EthApiServer` trait — additive.

### Build Order
Changes must be applied in dependency order:
1. `app` — Receipt re-export (leaf dependency)
2. `state` — BlockStorage trait (depends on `app`)
3. `app-evm` — Visibility changes + pending_receipts + store_finalized_block (depends on `state`)
4. `state-reth` — BlockStorage MDBX impl (depends on `state` + `app-evm`)
5. `rpc-eth` — New endpoints + EthRpcContext generic (depends on `state` + `app-evm`)
6. `whirlpool-node` — PersistingFinalizationSink wiring (depends on all above)

---

## S5 — Risk Assessment

### Implementation Risks

| Risk | Severity | Mitigation | Status |
|------|----------|------------|--------|
| `build_header_from_evm_block` reconstruction may lose unmapped fields in round-trip | Medium | INV-3 + TC-UNK-02 test validates fidelity; any drift caught before merge | Open |
| `pending_receipts` timing: propose() without subsequent finalization leaves stale cache | Low | INV-6 + `take().unwrap_or_default()` makes next finalization safe; stale receipts are at worst empty | Mitigated |
| `EthRpcContext` generic parameter change (`<S>` → `<S, B>`) touches all RPC construction sites | Low | Single call site in `whirlpool-node/src/main.rs`; same `RethStateDb` satisfies both bounds | Mitigated |
| MDBX write failure during finalization could lose block persistence | Medium | BLK-11 (deferred): log + continue policy; consensus proceeds; block can be re-finalized | Deferred |
| Thread contention on `Arc<RwLock<RethStateDb>>` between finalization writes and RPC reads | Low | MDBX handles concurrency internally (LMDB model); RwLock is for Rust borrow checker, not contention | Mitigated |

### Biggest Assumption
The existing `build_header_from_evm_block()` and `decode_transactions()` functions in `app-evm` are sufficient for the EvmBlock→MDBX storage conversion without needing new encoding/decoding logic. If these functions have gaps (e.g., missing fields, incompatible types), the storage layer would need additional conversion code.

**Evidence supporting assumption**: Both functions are already used in the EVM execution path and handle the full EvmBlock↔Header/Transaction lifecycle. INV-3 and TC-UNK-02 will validate this during implementation.

### Remaining Unknowns

| ID | Description | Resolution Plan |
|----|-------------|----------------|
| TC-UNK-01 | Receipt timing edge case (propose without finalization) | Test during app-evm impl; INV-6 covers safety |
| TC-UNK-02 | EvmBlock reconstruction fidelity after round-trip | Dedicated test in state-reth; INV-3 covers requirement |
| TC-UNK-03 | MDBX write failure handling (BLK-11) | MVP: log and continue; INV-7 covers consensus safety |
| TC-UNK-04 | Missing ephemeral block at finalization time | Test during whirlpool-node integration; log and skip |
