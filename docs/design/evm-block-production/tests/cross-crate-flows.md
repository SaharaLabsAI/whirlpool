# Cross-Crate Flow Test Contracts

Each row is a TDD-ready contract for architecture-level flows. Contracts marked `[PROPOSED]` are intentional blockers/revise targets until the relevant seam exists.

| Flow | Test case ID | Label | Invariant ref | Entry -> Exit | Preconditions | Actions | Assertions / Oracle | Mock/Stub requirements | Priority | Status |
|---|---|---|---|---|---|---|---|---|---|---|
| Block Proposal | `XFLOW-001` | `[GROUNDED]` | INV-06, INV-07 | `ConsensusApp::propose` -> `Option<EvmBlock>` | Adapter wraps `EvmApplication`; parent block available. | Trigger propose through adapter with current runtime wiring. | Returns `Some(block)`; `transactions=[]`; `transactions_root==EMPTY_ROOT_HASH`; `receipts_root==EMPTY_ROOT_HASH`; `gas_used==0`; `timestamp==parent.timestamp+12`. | `NoopTxSource`; in-memory state DB. | High | Active |
| Block Verification | `XFLOW-002` | `[GROUNDED]` | INV-02, INV-03 | `ConsensusApp::verify` -> `Result<(), ConsensusError>` | Valid block from `propose`; tampered copy with altered `state_root`. | Verify valid block, then verify tampered block via adapter. | Valid block accepted; tampered state-root block rejected as `InvalidBlock`; verify path leaves canonical root unchanged in current implementation. | `NoopTxSource`; in-memory state DB. | High | Active |
| State Commitment | `XFLOW-003` | `[PROPOSED]` | INV-04, INV-05 | finalize event -> canonical state commit | Finalized block exists; commit-ready artifact bound to block (missing seam). | Emit finalize event and invoke finalize->commit handoff. | Canonical commit applies effects exactly once; failed commit path preserves pre-finalize state exactly. | Requires finalize->commit seam and artifact store (currently missing). | High | Blocked |
| Node Startup Wiring | `XFLOW-004` | `[GROUNDED]` | INV-01 (precondition sentinel) | node `main` -> engine running | Node startup path reachable; runtime network/provider build succeeds. | Start node wiring path and inspect injected dependencies at boundary. | Runtime currently injects `NoopTxSource`; this keeps INV-01 preconditions (`>=1` valid pending tx) unsatisfied at node boundary and is tracked as a gap sentinel, not an invariant pass. | Stub/mock network allowed; use real config structs. | High | Revise |
| Block Finalization | `XFLOW-005` | `[GROUNDED]` | INV-05 (gap sentinel) | `ConsensusEvent::Finalized` -> sink side effect | `FinalizationSink` with shared `AtomicU64`; finalized event payload available. | Call sink `handle(Finalized { .. })`. | Finalized height is stored atomically; lack of canonical commit side effect is explicitly recorded as an INV-05 gap (does not satisfy commit atomicity). | Test block fixture; no state commit path invoked. | Medium | Revise |
| Non-Empty Proposal Visibility | `XFLOW-006` | `[PROPOSED]` | INV-01, INV-06 | non-noop tx source -> proposed block artifacts | Concrete tx source returns >=1 executable tx; execution path exists (currently missing). | Propose with non-empty tx source and inspect block/result artifacts. | Proposed block includes txs and non-empty execution-derived artifacts; roots align with executed txs. | Requires concrete tx source + execution integration. | High | Blocked |
| Non-Empty Determinism | `XFLOW-007` | `[PROPOSED]` | INV-07 | same state+pending set -> deterministic proposal | Deterministic tx ordering policy is defined (currently unknown). | Run propose twice with identical parent/state/tx input. | Produced blocks are byte-identical for deterministic fields. | Requires deterministic tx source policy contract. | Medium | Revise |

## Pseudo-code outlines

```rust
// XFLOW-001 [GROUNDED]
let genesis = adapter.genesis().await;
let block = adapter.propose(&genesis, 1).await.expect("Some block");
assert!(block.transactions.is_empty());
assert_eq!(block.gas_used, 0);
assert_eq!(block.transactions_root, EMPTY_ROOT_HASH.0);
assert_eq!(block.receipts_root, EMPTY_ROOT_HASH.0);
assert_eq!(block.timestamp, genesis.timestamp + 12);
```

```rust
// XFLOW-003 [PROPOSED] BLOCKER
let finalized = consensus_finalize(block);
let commit_result = finalize_commit_handoff(finalized);
assert!(commit_result.is_ok());
assert_commit_applied_once(finalized.block_id);
```

```rust
// XFLOW-006 [PROPOSED] BLOCKER
let tx_source = FixtureTxSource::with_pending(vec![valid_tx_bytes()]);
let app = build_app_with_tx_source(tx_source);
let (block, exec) = app.propose(&parent, next_height).await?;
assert!(!block.transactions.is_empty());
assert_eq!(block.state_root, exec.state_root);
assert_ne!(block.transactions_root, EMPTY_ROOT_HASH.0);
```
