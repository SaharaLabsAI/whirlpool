# Design Phase Digest

## Verdict: PASS

## Executive Summary
Design for persistent mempool storage in Whirlpool is complete and internally consistent. The design introduces a new `mempool` crate using raw libmdbx-rs for MDBX-backed transaction persistence, extends the `TxSource` trait with a `push()` method, generifies `EthRpcContext` to use trait objects, and wires everything through `whirlpool-node`.

## Key Design Decisions
1. **Raw libmdbx-rs** (not reth-db) — avoids vendor coupling, Tables enum constraint
2. **TxSource trait extension** — adds `push(&self, tx: Vec<u8>)`, all impls in-tree
3. **Auto-increment u64 keys** — FIFO ordering, no decode-on-insert cost
4. **Drain-on-pending** — matches current InMemoryTxPool semantics exactly
5. **Trait object generification** — `Arc<dyn TxSource + Send + Sync>` for EthRpcContext (matches EvmApplication)
6. **New mempool crate** — PersistentTxPool + MempoolStore, clean separation
7. **Crash window accepted** — proposed-but-not-finalized txs lost on crash (MVP scope)

## Crates Affected
| Crate | Change Type | Summary |
|---|---|---|
| mempool | NEW | PersistentTxPool, MempoolStore (MDBX persistence) |
| app | MODIFIED | TxSource trait gets push(), InMemoryTxPool updated |
| rpc-eth | MODIFIED | EthRpcContext uses Arc<dyn TxSource> |
| whirlpool-node | MODIFIED | Wires PersistentTxPool with storage path |
| app-evm | UNCHANGED | Already uses dyn TxSource |

## Implementation Ordering
1. TxSource trait extension (app)
2. Update InMemoryTxPool + NoopTxSource (app)
3. EthRpcContext generification (rpc-eth)
4. New mempool crate with MempoolStore + PersistentTxPool
5. Node wiring (whirlpool-node)
6. Integration tests
7. End-to-end validation

## Test Coverage
- 67 test cases across unit/integration/property/e2e
- All 5 FLOWS.md flows validated
- All INTENT success criteria mapped to tests
- Critical path: 4 must-pass tests for MVP

## Blockers
- 0 active blockers
- 3 accepted warnings (crash window, path misconfiguration, dedup complexity)

## Documents Produced
- INTENT.md, STRATEGY.md, SHARED_CONTEXT.md, EXPLORATION.md, EXPLORATION_DIGEST.md
- CRATES.md, WORKSPACE.md, DOMAINS.md, FLOWS.md, TESTS.md, BLOCKERS.md
- INDEX.md, SUMMARY.md
- crates/mempool.md, crates/app.md, crates/rpc-eth.md, crates/whirlpool-node.md

## Gate: AUTO-APPROVED (auto_approve=true)
Proceeding to Phase 2 (PROVE).
