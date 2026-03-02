# Block Finalization

## Trigger

Consensus emits `ConsensusEvent::Finalized { block, height, proof }` into event sink path.

## Stages

1. Consensus runtime produces finalized event with block + metadata.
2. `FinalizationSink` receives event through `EventSink::handle` contract.
3. Sink updates shared `AtomicU64` finalized height.
4. Node can observe finalized-height progress from shared atomic state.
5. Canonical timing rule: `propose()` remains speculative; [PROPOSED] canonical commit should occur only after this finalization step via finalize->commit seam (`BLOCKER`).

## Outputs

- Grounded output: updated finalized-height signal.
- [PROPOSED] canonical commit trigger output: finalized block becomes eligible for finalize->commit handoff.
- `UNKNOWN` output: canonical state commit completion bound to finalized block.

## Stage ownership

- Finalization event schema: `consensus`.
- Event handling side effects: `consensus-simplex` (`FinalizationSink`).
- Runtime observation of progress: `whirlpool-node`.
- Canonical commit ownership after finalization: `UNKNOWN` integration seam.

## Handoff contracts

- `ConsensusEvent::Finalized` -> `EventSink::handle` is grounded.
- Sink side effect contract: atomic finalized-height write is grounded.
- [PROPOSED] finalize->commit seam is the only canonical commit path.
- `BLOCKER`: no grounded contract carries finalized block effects into `state::commit` path.

## Error propagation

- Current sink path shows atomic update/logging, with no grounded commit-stage failure path.
- `UNKNOWN`: how finalization/commit errors should surface to node liveness or consensus telemetry.

## Pseudo-code sketch

```rust
fn handle_event(event: ConsensusEvent<EvmBlock>) {
    match event {
        ConsensusEvent::Finalized { block, height, proof: _ } => {
            finalized_height.store(height, Ordering::SeqCst);
            // [PROPOSED] only here trigger canonical commit handoff
            // commit_finalized_block(block)
        }
        _ => {}
    }
}
```

## Open questions / TODOs

- `BLOCKER`: define finalize -> commit contract across `consensus`/`app`/`state` seams.
- `UNKNOWN`: durability and idempotency semantics for repeated finalization signals.
- `UNKNOWN`: reorg/fault handling once commit path exists.
- [PROPOSED] add integration tests that pair finalized event observation with commit-side verification.
