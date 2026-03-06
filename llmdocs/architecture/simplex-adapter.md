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
- `AppAdapter`: maps vendor `Application/VerifyingApplication/Reporter` callbacks to consensus traits. Internally tracks finalized blocks via `Arc<RwLock<HashMap<Digest, B>>>` shared across all clones, enabling the reporter actor to find blocks stored by the batcher/automaton actor during propose/verify/genesis.
- `CommonwareEngine`: sealed constructor/startup for mailbox, adapter, sink, and simplex engine.
- `FinalizationSink`: `EventSink` implementation tracking finalized height.

## Clone Semantics
`AppAdapter::Clone` clones the `Arc` to the shared `finalized_blocks` map. All clones (batcher, voter/reporter) operate on the same block store. `remember_block()` is async (acquires write lock).

## Data Flow
- propose: vendor -> `AppAdapter::propose` -> `ConsensusApp::propose` -> `remember_block()`
- verify: vendor -> `AppAdapter::verify` -> `ConsensusApp::verify` -> `remember_block()`
- genesis: vendor -> `AppAdapter::genesis` -> `ConsensusApp::genesis` -> `remember_block()`
- finalized event: vendor `Activity::Finalization` -> `AppAdapter::report` -> removes block from shared `finalized_blocks` -> `ConsensusEvent::Finalized` -> `EventSink::handle` -> ack

## Status
Complete. `CommonwareBlock` moved to `traits.rs`; imports use canonical `::traits::` paths. `finalized_blocks` uses `Arc<RwLock<HashMap>>` for cross-clone sharing.
