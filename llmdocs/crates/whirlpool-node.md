# whirlpool-node: EVM Consensus Binary

## Summary

`whirlpool-node` runs Commonware consensus with `EvmApplication` on Sahara Chain.

Location: `crates/whirlpool-node/`

## Dependencies (All Required)

- **consensus**: Core trait layer (Block, ConsensusEngine, ConsensusApp)
- **consensus-simplex**: Sealed adapter (CommonwareEngine, FinalizationSink, CommonwareConfig)
- **whirlpool-node** (lib): Shared exports (config, EmptyBlock, EmptyBlockApp)
- **app-evm**: EVM executor and state provider
- **app**: `ApplicationAdapter` and `InMemoryTxPool` for tx sourcing
- **state**: In-memory state database
- **revm**: EVM execution engine
- **alloy-primitives**: EVM primitive types
- **p2p-commonware**: Network provider bridge
- **commonware-runtime**: Tokio-based runtime

## main.rs Structure

File: `crates/whirlpool-node/src/main.rs`

1. **Initialization**: Tracing, constants, finalization sink, runtime setup.
2. **State Provider**: `TestStateDb` wraps `InMemoryStateDb` and implements `StateProvider` + `revm::Database`.
3. **Application Wiring**:
   - Build chain spec and `WhirlpoolEvmConfig`.
   - Create `Arc<InMemoryTxPool>` via `InMemoryTxPool::new()`.
   - Pass tx pool into `EvmApplication::new(...)`.
   - Wrap with `ApplicationAdapter`, then start `CommonwareEngine`.

## Wiring Change

- Node now uses `InMemoryTxPool` (`crates/whirlpool-node/src/main.rs:15`, `crates/whirlpool-node/src/main.rs:130`).
- Previous `NoopTxSource` wiring is removed from the EVM binary path.

## Use Cases

- Production consensus for Sahara EVM blocks
- Contract execution with deterministic state transitions
- Finalization-driven state root progression
- Networked block propagation with EVM validation

## Related Documentation

- `llmdocs/crates/app.md`
- `llmdocs/crates/app-evm.md`
- `llmdocs/architecture/whirlpool-node.md`
- `llmdocs/crates/whirlpool-node-simple.md`
