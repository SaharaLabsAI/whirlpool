# whirlpool-node Unit Test Contracts

| Test name | Test case ID | Label | Invariant ref | Preconditions | Actions | Assertions / Oracle | Mock/Stub requirements | Priority | Status |
|---|---|---|---|---|---|---|---|---|---|
| `startup_wires_noop_tx_source_in_current_runtime` | `NODE-U-001` | `[GROUNDED]` | INV-07 (INV-01 precondition sentinel) | Startup composition path is reachable in test harness. | Build startup dependencies as in `main` wiring. | Injected tx source is `NoopTxSource`; deterministic noop wiring is preserved and INV-01 preconditions (>=1 valid pending tx) are explicitly unsatisfied at this boundary. | Stub network/provider context; real config structs. | High | Active |
| `startup_composes_evm_app_adapter_engine_without_type_mismatch` | `NODE-U-002` | `[GROUNDED]` | INV-06 | Test can instantiate state DB, EVM config, adapter, sink, and engine config. | Construct `EvmApplication -> ApplicationAdapter -> CommonwareEngine` chain. | Composition succeeds and engine startup path is callable. | Mock network provider acceptable; real application objects. | High | Active |
| `finalization_sink_updates_atomic_height_on_finalized_event` | `NODE-U-003` | `[GROUNDED]` | INV-05 (gap sentinel) | `FinalizationSink` has shared `AtomicU64` height initialized to 0. | Call `handle(ConsensusEvent::Finalized { height=h, .. })`. | Atomic height equals `h` after event handling; absence of commit handoff side effect is explicitly recorded as current INV-05 gap. | Finalized event fixture. | High | Revise |
| `runtime_wiring_supports_non_noop_tx_source_for_execution_visibility` | `NODE-U-004` | `[PROPOSED]` | INV-01 | Concrete tx source implementation exists and is injectable at startup boundary. | Wire node with concrete tx source and start proposal path. | Proposals can observe pending txs; runtime no longer hardcodes noop ingress. | Requires concrete `TxSource` implementation and startup wiring update. | High | Blocked |
| `finalization_event_triggers_commit_handoff_contract` | `NODE-U-005` | `[PROPOSED]` | INV-05 | Finalize->commit callback seam is defined across node/app/state boundaries. | Emit finalized event and observe commit handoff invocation. | Commit handoff invoked exactly once per finalized block and failure path is surfaced. | Requires explicit finalize callback integration (currently absent). | High | Blocked |

## Pseudo-code outlines

```rust
// NODE-U-001 [GROUNDED]
let deps = build_node_startup_deps();
let app = deps.build_app();
assert_tx_source_is_noop(&app);
```

```rust
// NODE-U-003 [GROUNDED]
let height = Arc::new(AtomicU64::new(0));
let sink = FinalizationSink::<EvmBlock>::new(height.clone());
sink.handle(finalized_event(7)).await;
assert_eq!(height.load(Ordering::SeqCst), 7);
```

```rust
// NODE-U-005 [PROPOSED] BLOCKER
emit_finalized(block);
assert_handoff_invoked(block.id());
assert_commit_applied_once(block.id());
```
