# Simplex Adapter Bridge

The adapter crate translates Commonware Simplex APIs into Whirlpool consensus traits.

## Interface/Implementation Split
- Interface module: `crates/consensus-simplex/src/traits.rs`
  - `CommonwareBlock`
- Implementation modules:
  - `crates/consensus-simplex/src/adapter.rs`
  - `crates/consensus-simplex/src/engine.rs`
  - `crates/consensus-simplex/src/mailbox.rs`
  - `crates/consensus-simplex/src/sink.rs`
  - `crates/consensus-simplex/src/config.rs`

## Canonical Imports
- Core traits: `consensus::traits::{Block, ConsensusApp, EventSink, ConsensusEngine}`
- Adapter trait: `consensus_simplex::traits::CommonwareBlock`

## Core Types
- `CommonwareBlock`: super-trait requiring `consensus::traits::Block + commonware_consensus::Block + Clone`.
- `CommonwareConfig`: simplex timing and buffer configuration.
- `AppAdapter`: maps vendor `Application/VerifyingApplication/Reporter` callbacks to consensus traits.
- `CommonwareEngine`: sealed constructor/startup for mailbox, adapter, sink, and simplex engine.
- `FinalizationSink`: `EventSink` implementation tracking finalized height.

## Data Flow
- propose: vendor -> `AppAdapter::propose` -> `ConsensusApp::propose`
- verify: vendor -> `AppAdapter::verify` -> `ConsensusApp::verify`
- finalized event: vendor `Update::Block` -> `ConsensusEvent::Finalized` -> `EventSink::handle` -> ack

## Status
Complete. `CommonwareBlock` moved to `traits.rs`; imports use canonical `::traits::` paths.
