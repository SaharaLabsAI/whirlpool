# Oracle Self-Check Pack — Persistent Block Storage & History Queries

## Document Inventory (13 docs + 6 crate contracts)

| Doc | Lines | Purpose |
|---|---|---|
| INTENT.md | 33 | 5 success criteria (SC-1..SC-5), scope constraints |
| STRATEGY.md | 282 | 3 streams, 5 key decisions, open questions |
| CRATES.md | 175 | Per-crate change size + new file listing |
| WORKSPACE.md | 281 | Dependency graph, build order |
| DOMAINS.md | 684 | 5 domains, cross-domain wiring, interface specs |
| FLOWS.md | 291 | 4 architecture flows with [EXISTING]/[NEW] annotations |
| TESTS.md | 94 | Test strategy, SC mapping, unit/integration/flow tests |
| BLOCKERS.md | 78 | 0 active, 7 deferred, 3 resolved |
| EXPLORATION.md | 95 | Raw findings |
| EXPLORATION_DIGEST.md | 21 | Compact exploration summary |
| SHARED_CONTEXT.md | 33 | Architecture context |
| RISK_TRIAGE.md | 42 | 5 risks classified |
| crates/state.md | 107 | BlockStorage trait contract |
| crates/state-reth.md | 241 | MDBX BlockStorage impl contract |
| crates/app-evm.md | 227 | Receipt lifecycle + finalization hook |
| crates/rpc-eth.md | 321 | eth_getBlock* endpoints |
| crates/whirlpool-node.md | 146 | PersistingFinalizationSink wiring |
| crates/consensus-simplex.md | 57 | No changes (justified) |

## Cross-Doc Consistency Checks Required

1. **Crate coverage**: Every crate in CRATES.md has a README in docs/crates/
2. **Domain coverage**: Every crate in CRATES.md appears in at least one DOMAINS.md domain
3. **Flow grounding**: Every flow step in FLOWS.md references an interface defined in a crate README
4. **Test mapping**: Every SC in INTENT.md maps to at least one test in TESTS.md
5. **Interface consistency**: Method signatures in crate READMEs match DOMAINS.md interface specs
6. **BLOCKER reconciliation**: No unresolved BLOCKERs that would prevent implementation
7. **Dependency consistency**: WORKSPACE.md deps match crate README "new deps" sections
8. **No contradictions**: STRATEGY.md decisions consistent with DOMAINS.md and crate READMEs

## Known Issues From Prior Gates
- D5 gate found 3 inconsistencies (domain count, receipt method, MVP scope) — all fixed
- FLOWS.md and crate contracts written in parallel after D5 fixes

## Success Criteria Traceability
- SC-1 (atomic persistence) → state.store_block → state-reth MDBX impl → TC-SR-01, TC-SR-07
- SC-2 (auto finalization) → app-evm.store_finalized_block → PersistingFinalizationSink → TC-INT-01
- SC-3 (getBlockByNumber) → rpc-eth handler → TC-RPC-02, TC-RPC-05, TC-RPC-07
- SC-4 (getBlockByHash) → rpc-eth handler → TC-RPC-04
- SC-5 (node wiring) → whirlpool-node → TC-INT-02

## Open Questions (from TESTS.md)
- TC-UNK-01: Receipt timing edge case (propose without finalization) — UNKNOWN
- TC-UNK-02: EvmBlock reconstruction fidelity — UNKNOWN
- TC-UNK-03: MDBX write failure handling — BLOCKER
- TC-UNK-04: Missing ephemeral block at finalization — UNKNOWN
