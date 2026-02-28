# whirlpool-node-simple: Non-EVM Consensus Binary

## Summary

`whirlpool-node-simple` is a consensus node binary for minimal/non-EVM consensus experiments. It runs the Commonware consensus engine with `EmptyBlockApp` (pure consensus without EVM execution).

Location: `crates/whirlpool-node-simple/`

## Dependencies

- **consensus**: Core trait layer (Block, ConsensusEngine, ConsensusApp)
- **consensus-simplex**: Sealed adapter (CommonwareEngine, FinalizationSink, CommonwareConfig)
- **whirlpool-node**: Library exports (EmptyBlockApp, EmptyBlock, config)
- **p2p-commonware**: Commonware network provider bridge
- **commonware-runtime**: tokio-based async runtime (via commonware)

## Architecture: No Feature Gates

Unlike the original whirlpool-node, `whirlpool-node-simple` has no feature gates. It unconditionally uses:
- `EmptyBlockApp`: Simple consensus app with identity block verification (parent/height/genesis checks only)
- `CommonwareEngine`: Sealed consensus wiring from consensus-simplex
- `CommonwareNetworkProviderBuilder`: Real network provider (no mocks)

## main.rs Structure

File: `crates/whirlpool-node-simple/src/main.rs`

1. **Initialization** (lines 23–32): tracing_subscriber setup
2. **Consensus wiring** (lines 34–37): FinalizationSink, EmptyBlockApp
3. **Runtime launch** (lines 39–41): tokio::Runner::default().start()
4. **Network setup** (lines 43–54): ed25519 signer, CommonwareNetworkProviderBuilder
5. **Engine construction** (lines 56–63): CommonwareEngine with identity app, config
6. **Engine execution** (lines 64–69): engine.start() loop with finalization tracking

## Test Status

- **Unit tests**: 0 (binary crate, no lib)
- **Integration tests**: None
- **Binary builds**: ✓ (target/debug/whirlpool-node-simple, 145M)

## Use Cases

- **Consensus research**: Test consensus engine without EVM overhead
- **Network testing**: Verify p2p discovery and message propagation
- **Performance benchmarks**: Measure throughput without execution layer
- **Minimal reference implementation**: Pure consensus without application complexity

## Related Documentation

- `crates/whirlpool-node.md`: EVM-enabled variant (same crate, different binary, using app-evm)
- `architecture/whirlpool-node.md`: Shared library (EmptyBlock, EmptyBlockApp, config)
- `guides/whirlpool-node-components.md`: How to extend or modify the node
