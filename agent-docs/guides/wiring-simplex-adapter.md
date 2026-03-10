# Wiring the Simplex Adapter

This guide describes wiring against the post-refactor interface boundaries.

## Canonical Imports
- `consensus::traits::{Block, ConsensusApp, EventSink, ConsensusEngine}`
- `consensus_simplex::traits::CommonwareBlock`
- `p2p_commonware::traits::CommonwareTransport` (when using dedicated simplex channels)

## 1) Block Contract
Your block type must satisfy:
- `consensus::traits::Block`
- `commonware_consensus::Block`
- `Clone`
Together this is `consensus_simplex::traits::CommonwareBlock`.

## 2) App Contract
Implement `ConsensusApp` with `genesis`, `propose`, and `verify`.

## 3) Sink Contract
Implement `EventSink` to consume `ConsensusEvent` values.

## 4) Engine Wiring
Create `CommonwareEngine::new(app, sink, config)` and call `start()`.

## 5) Network Wiring
Use `CommonwareNetworkProviderBuilder` for multiplexed provider mode.
For simplex-dedicated channels, depend on `CommonwareTransport::start_per_channel`.

## Migration Rule
Use `::traits::` canonical paths across crates; trait imports from crate roots are non-canonical.
