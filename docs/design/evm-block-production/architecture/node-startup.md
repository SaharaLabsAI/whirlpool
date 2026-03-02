# Node Startup

## Trigger

Process entrypoint starts `whirlpool-node` main runtime.

## Stages

1. Initialize tracing/runtime execution context.
2. Build signer and network provider via commonware wiring.
3. Build consensus configuration (`CommonwareConfig`).
4. Construct state DB and EVM config (`WhirlpoolEvmConfig`).
5. Inject tx source (currently `NoopTxSource`) into `EvmApplication`.
6. Wrap `EvmApplication` in `ApplicationAdapter`.
7. Construct `CommonwareEngine` with app, sink, config, network provider.
8. Start engine (`ConsensusEngine::start`).
9. Current simplex engine start path includes explicit STUB/simulated behavior.

## Outputs

- Running consensus engine handle on successful startup.
- Shared finalized-height progress signal via `FinalizationSink` + atomic counter.
- Node process failure if engine start fails (fatal boundary in current startup path).

## Stage ownership

- Composition and dependency injection: `whirlpool-node`.
- App lifecycle contract: `app`.
- EVM app implementation: `app-evm`.
- State backend: `state`.
- Consensus runtime lifecycle: `consensus-simplex` implementing `consensus` traits.

## Handoff contracts

- Node -> app: `EvmApplication::new(config, db, tx_source)`.
- Node -> adapter: `ApplicationAdapter::new(app)`.
- Node -> consensus engine: `CommonwareEngine::new(...).start()` via `ConsensusEngine` trait.
- Node -> finalization sink: inject shared atomic finalized height.

## Error propagation

- Engine startup failures surface through `ConsensusEngine::start` result.
- Simplex startup path maps failures into `ConsensusError::Other` in current implementation.
- Node startup treats start failure as fatal in current runtime wiring.

## Pseudo-code sketch

```rust
fn node_startup() -> Result<(), NodeError> {
    let network = build_network_provider()?;
    let cfg = build_consensus_config();
    let db = build_state_db();
    let tx_source = Arc::new(NoopTxSource); // current grounded wiring
    let app = EvmApplication::new(evm_cfg(), db, tx_source);
    let adapter = ApplicationAdapter::new(app);
    let sink = FinalizationSink::new(shared_finalized_height());
    let engine = CommonwareEngine::new(cfg, adapter, sink, network);
    engine.start()?;
    Ok(())
}
```

## Open questions / TODOs

- `BLOCKER`: swap `NoopTxSource` for concrete source to enable non-empty proposal flow.
- `UNKNOWN`: non-stub simplex startup behavior details in full production runtime path.
- [PROPOSED] add startup-time wiring assertions for tx source and finalization commit coupling.
- [PROPOSED] define startup config surface for tx ingress/backpressure.
