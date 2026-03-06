# Design Phase Digest — Persistent Block Storage & History Queries

**Instance**: `persistent-blocks-history-20260306-1500`
**Phase**: 1 (DESIGN) — Complete
**Sub-phases**: D1–D9 all complete

## Verdict: PASS

The design phase produced a complete, internally-consistent document set covering persistent block storage and history query capabilities for the Whirlpool consensus framework.

## Key Decisions Made

1. **BlockStorage trait in `state` crate** — new trait with 4 methods (store_block, get_block_by_number, get_block_by_hash, get_receipts_by_block), MDBX implementation in `state-reth`
2. **Persistence at application layer** — `EvmApplication` owns the persistence hook, NOT consensus-simplex (which remains generic over `B: Block`)
3. **PersistingFinalizationSink wrapper** — node-level composite sink in `whirlpool-node` that calls `store_finalized_block()` on finalization events
4. **Reuse existing MDBX tables** — Headers, BlockBodyIndices, Transactions, Receipts etc. already created by `init_db()`
5. **Receipt fallback DEFERRED** — `eth_getTransactionReceipt` persistent fallback is post-MVP (BLK-3)

## Document Set (19 files)

| Doc | Lines | Purpose |
|---|---|---|
| INTENT.md | 41 | Success criteria SC-1..SC-5 |
| STRATEGY.md | 282 | 3 streams, 5 key decisions |
| CRATES.md | 175 | Per-crate change descriptions |
| WORKSPACE.md | 284 | Dependency graph + build order |
| DOMAINS.md | 692 | 5 domains + cross-domain wiring |
| FLOWS.md | 291 | 4 architecture flows |
| TESTS.md | 94 | 22 unit + 2 integration + 4 flow tests |
| BLOCKERS.md | 85 | 0 active, 8 deferred, 3 resolved |
| INDEX.md | 70 | Loading tiers + reading guide |
| SUMMARY.md | 37 | Executive summary |
| crates/*.md | 7 files | Per-crate contracts |

## Oracle Self-Check Results

Oracle found 6 consistency issues — **all fixed**:
1. ✅ INTENT.md now has explicit SC-1..SC-5 labels
2. ✅ Created missing `docs/crates/app.md`
3. ✅ DOMAINS.md now notes unaffected crates
4. ✅ FLOWS.md grounding fixed (PersistingFinalizationSink, method naming)
5. ✅ WORKSPACE.md updated with missing deps (state-reth→app-evm, rpc-eth→app-evm)
6. ✅ Receipt fallback consistently DEFERRED, MDBX write failure added as BLK-11

## Blockers Status

- **0 active blockers** — nothing prevents implementation
- **8 deferred** — all have mitigations noted, none block MVP
- **3 resolved** — closed during exploration/design

## Open Unknowns (from TESTS.md)

- TC-UNK-01: Receipt timing edge case (propose without finalization) — resolve during impl
- TC-UNK-02: EvmBlock reconstruction fidelity — resolve with round-trip tests
- TC-UNK-03: MDBX write failure handling — deferred as BLK-11, MVP logs and continues
- TC-UNK-04: Missing ephemeral block at finalization — resolve during impl

## Next Steps (pending approval)

- **Phase 2 (PROVE)**: Validate intent split, generate acceptance criteria + invariants
- **Phase 3 (PLAN)**: Generate sisyphus execution plan from design docs
- **Phase 4 (EXECUTE)**: Implement via `/start-work`
