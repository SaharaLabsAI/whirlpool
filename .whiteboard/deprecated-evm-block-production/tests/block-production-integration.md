# Block Production Integration Test Contracts

| Test name | Test case ID | Label | Invariant ref | Preconditions | Actions | Assertions / Oracle | Mock/Stub requirements | Priority | Status |
|---|---|---|---|---|---|---|---|---|---|
| `consensus_adapter_propose_verify_roundtrip_empty_path` | `BP-INT-001` | `[GROUNDED]` | INV-06, INV-07 | Engine-side app boundary wraps current MVP `EvmApplication` with noop tx source. | Execute `genesis -> propose -> verify` through consensus-facing adapter. | Proposal yields empty tx block with empty roots/zero gas; verify succeeds for untampered block. | Deterministic state DB fixture; noop tx source. | High | Active |
| `verify_rejects_tampered_state_root_via_consensus_invalid_block` | `BP-INT-002` | `[GROUNDED]` | INV-02 | Candidate block from proposal path; cloned block with altered `state_root`. | Call adapter `verify` on tampered block. | Returns `ConsensusError::InvalidBlock(..)` due to mapped state-root mismatch. | Same fixtures as BP-INT-001. | High | Active |
| `failed_verification_does_not_mutate_canonical_state_root` | `BP-INT-003` | `[GROUNDED]` | INV-03, INV-04 | Canonical state root captured before verify failure injection. | Run verify with tampered state-root block, then query canonical root again. | Root after failure equals root before failure in current read-only verify path. | In-memory DB fixture accessible before/after verify call. | High | Active |
| `finalized_event_updates_height_without_canonical_commit` | `BP-INT-004` | `[GROUNDED]` | INV-05 (gap sentinel) | Finalization sink wired with shared `AtomicU64`; state commit seam absent. | Emit finalized event in integration harness. | Finalized height updates; no canonical state commit side effect observed; this is tracked as an explicit INV-05 gap sentinel, not a commit-atomicity pass. | Finalized event fixture; state root probe helper. | High | Revise |
| `startup_to_engine_path_uses_runtime_noop_tx_source` | `BP-INT-005` | `[GROUNDED]` | INV-01 (precondition sentinel) | Node startup wiring path is executable in integration harness. | Build runtime components and inspect app injection chain. | Wiring shows `NoopTxSource` in current runtime graph; INV-01 preconditions (`>=1` valid pending tx) remain unsatisfied in this path. | Network/provider stubs allowed. | Medium | Revise |
| `end_to_end_finalize_commit_applies_effects_exactly_once` | `BP-INT-006` | `[PROPOSED]` | INV-05 | Finalize->commit seam exists and commit-ready artifact is bound to finalized block. | Drive `propose -> finalize -> commit` and repeat finalize signal for same block. | Canonical effects applied exactly once; duplicate finalize is idempotent/no-op. | Requires finalize callback and artifact storage ownership seam. | High | Blocked |
| `proposal_with_non_empty_tx_source_produces_execution_visible_block` | `BP-INT-007` | `[PROPOSED]` | INV-01, INV-06 | Runtime wired with concrete tx source that returns executable tx bytes. | Drive proposal through consensus-facing boundary with non-empty pending set. | Produced block includes txs and execution-derived roots/gas artifacts. | Requires concrete tx source + execution implementation in app-evm. | High | Blocked |

## Pseudo-code outlines

```rust
// BP-INT-001 [GROUNDED]
let genesis = adapter.genesis().await;
let block = adapter.propose(&genesis, 1).await.expect("proposal");
assert!(block.transactions.is_empty());
adapter.verify(&genesis, &block).await.expect("verify");
```

```rust
// BP-INT-004 [GROUNDED]
let before = canonical_state_root();
sink.handle(finalized_event(2)).await;
assert_eq!(finalized_height(), 2);
assert_eq!(canonical_state_root(), before);
```

```rust
// BP-INT-006 [PROPOSED] BLOCKER
let block = propose_non_empty_block()?;
finalize(block.clone())?;
finalize(block)?;
assert_commit_applied_once();
```
