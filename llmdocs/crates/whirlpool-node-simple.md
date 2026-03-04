# whirlpool-node-simple: Non-EVM Consensus Binary

## Summary
`whirlpool-node-simple` is a self-contained consensus node for non-EVM experiments.

Location: `crates/whirlpool-node-simple/`

## Canonical Trait Usage
- Core consensus traits are imported via `consensus::traits::*`.
- Block type satisfies `consensus::traits::Block` and vendor trait bounds needed by simplex.

## Architecture
- Local `EmptyBlock` and `EmptyBlockApp` implementations.
- `CommonwareEngine` from `consensus-simplex` handles sealed wiring.
- Real Commonware networking via `p2p-commonware` provider.

## Crate Structure
- `src/lib.rs`: exports local app/block modules.
- `src/block.rs`: `EmptyBlock` with dual-trait conformance (`consensus::traits::Block` + commonware traits).
- `src/app.rs`: `EmptyBlockApp` implementing `ConsensusApp`.
- `src/main.rs`: runtime and engine startup wiring.

## Status
Complete for minimal consensus use cases.
