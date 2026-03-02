# app Unit Test Contracts

| Test name | Test case ID | Label | Invariant ref | Preconditions | Actions | Assertions / Oracle | Mock/Stub requirements | Priority | Status |
|---|---|---|---|---|---|---|---|---|---|
| `noop_tx_source_pending_is_empty` | `APP-U-001` | `[GROUNDED]` | INV-07 (INV-01 precondition sentinel) | `NoopTxSource` instance exists. | Call `pending()` multiple times. | Each call returns `Vec::new()`; output is deterministic for identical source state; empty pending set is explicitly treated as INV-01 precondition unsatisfied, not execution-visibility pass. | None. | High | Active |
| `adapter_genesis_passthrough_returns_inner_block` | `APP-U-002` | `[GROUNDED]` | INV-06 | Adapter wraps mock app with known genesis `EvmBlock`. | Call `ApplicationAdapter::genesis()`. | Returned block equals inner app genesis for all consensus-visible fields. | Mock `Application` with fixed genesis fixture. | High | Active |
| `adapter_propose_maps_ok_to_some_block` | `APP-U-003` | `[GROUNDED]` | INV-06 | Mock app `propose` returns `Ok((block, exec))`. | Call `ApplicationAdapter::propose(parent, height)`. | Adapter returns `Some(block)` and preserves block fields. | Mock `Application` returning fixture `(EvmBlock, ExecutionResult)`. | High | Active |
| `adapter_propose_maps_error_to_none_and_recovery_is_stable` | `APP-U-004` | `[GROUNDED]` | INV-04 | Mock app can produce one failing propose then one succeeding propose for same parent/height. | Call `propose` twice on same input (first fails, second succeeds). | First call returns `None`; second call returns `Some(expected_block)` identical to baseline success, showing no retained failure-side effects at adapter boundary. | Stateful mock `Application` (fail-once then success). | High | Active |
| `adapter_verify_maps_ok_to_ok_unit` | `APP-U-005` | `[GROUNDED]` | INV-02, INV-03 | Mock app `verify` returns `Ok(exec)`. | Call `ApplicationAdapter::verify(parent, block)`. | Adapter returns `Ok(())`. | Mock `Application` with successful verify result. | High | Active |
| `tx_source_pending_order_is_stable_for_same_snapshot` | `APP-U-006` | `[PROPOSED]` | INV-07 | Concrete non-noop tx source exists and supports snapshot-consistent reads. | Query `pending()` twice against unchanged pool snapshot. | Returned tx byte sequence is byte-identical and in the same order each time. | Requires concrete `TxSource` implementation (currently missing in runtime wiring). | Medium | Revise |
| `propose_to_finalize_artifact_boundary_is_explicit` | `APP-U-007` | `[PROPOSED]` | INV-05 | Proposal path exposes commit-ready artifact ownership contract. | Execute propose, then invoke finalize boundary handoff. | Artifact required for canonical commit is retained and addressable by finalized block identity. | Requires finalize->commit seam and artifact store (not grounded). | High | Blocked |

## Pseudo-code outlines

```rust
// APP-U-001 [GROUNDED]
let source = NoopTxSource;
let a = source.pending();
let b = source.pending();
assert!(a.is_empty());
assert_eq!(a, b);
```

```rust
// APP-U-004 [GROUNDED]
let app = MockApplication::fail_once_then_success(expected_block.clone());
let adapter = ApplicationAdapter::new(app);
assert!(adapter.propose(&parent, 1).await.is_none());
let recovered = adapter.propose(&parent, 1).await.expect("recovery");
assert_eq!(recovered, expected_block);
```

```rust
// APP-U-006 [PROPOSED] REVISE
let source = FixtureTxSource::from_ordered_pool(vec![tx_a(), tx_b()]);
let first = source.pending();
let second = source.pending();
assert_eq!(first, second);
```
