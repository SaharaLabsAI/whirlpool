# whirlpool-node-simple: Non-EVM Consensus Binary

## Summary

`whirlpool-node-simple` is a self-contained consensus node binary for minimal/non-EVM consensus experiments. It runs the Commonware consensus engine with `EmptyBlockApp` (pure consensus without EVM execution).

Location: `crates/whirlpool-node-simple/`

## Dependencies

- **consensus**: Core trait layer (Block, ConsensusEngine, ConsensusApp)
- **consensus-simplex**: Sealed adapter (CommonwareEngine, FinalizationSink, CommonwareConfig)
- **whirlpool-node**: Config constants only (NAMESPACE, BLOCK_INTERVAL, etc.)
- **p2p-commonware**: Commonware network provider bridge
- **commonware-runtime**: tokio-based async runtime (via commonware)
- **commonware-codec**: Codec traits (CodecRead, CodecWrite, EncodeSize)
- **commonware-consensus**: Heightable trait
- **bytes**: Buffer traits (Buf, BufMut)
- **sha2**: SHA-256 for block ID computation

## Architecture

Self-contained crate with local `EmptyBlock` and `EmptyBlockApp` types (previously shared from whirlpool-node). No feature gates. Unconditionally uses:
- `EmptyBlockApp` (local): Simple consensus app with identity block verification (parent/height/genesis checks only)
- `EmptyBlock` (local): Minimal block type with SHA-256 ID
- `CommonwareEngine`: Sealed consensus wiring from consensus-simplex
- `CommonwareNetworkProviderBuilder`: Real network provider (no mocks)

## Crate Structure

- `src/lib.rs`: `pub mod app; pub mod block;`
- `src/block.rs`: `EmptyBlock` struct with dual-trait conformance (consensus::Block + commonware traits)
- `src/app.rs`: `EmptyBlockApp` implementing `ConsensusApp<EmptyBlock>` with 5 verification rules
- `src/main.rs`: Binary entry point wiring consensus engine

## Test Status

- **Unit tests**: 18 (block.rs: 7, app.rs: 11) — embedded in local modules
- **Integration tests**: None
- **Binary builds**: ✓

## Use Cases

- **Consensus research**: Test consensus engine without EVM overhead
- **Network testing**: Verify p2p discovery and message propagation
- **Performance benchmarks**: Measure throughput without execution layer
- **Minimal reference implementation**: Pure consensus without application complexity

## Related Documentation

- `crates/whirlpool-node.md`: EVM-enabled variant using app-evm
- `architecture/whirlpool-node.md`: Node library (config exports)
- `guides/whirlpool-node-components.md`: How to extend or modify the node
