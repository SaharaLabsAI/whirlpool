# Simplex Adapter Bridge

The adapter crate translates Commonware Simplex APIs into Whirlpool consensus traits, and provides a payload relay for broadcasting proposed blocks to peers.

## Interface/Implementation Split
- Interface module: `crates/consensus-simplex/src/traits.rs`
  - `CommonwareBlock`
- Implementation modules:
  - `crates/consensus-simplex/src/adapter.rs`
  - `crates/consensus-simplex/src/engine.rs`
  - `crates/consensus-simplex/src/mailbox.rs`
  - `crates/consensus-simplex/src/receiver.rs`
  - `crates/consensus-simplex/src/sink.rs`
  - `crates/consensus-simplex/src/config.rs`

## Canonical Imports
- Core traits: `consensus::traits::{Block, ConsensusApp, EventSink, ConsensusEngine}`
- Adapter trait: `consensus_simplex::traits::CommonwareBlock`
- Relay types: `consensus_simplex::{PayloadRelayMessage, payload_receive_loop}`

## Core Types
- `CommonwareBlock`: super-trait requiring `consensus::traits::Block + commonware_consensus::Block + Clone`.
- `CommonwareConfig`: simplex timing, buffer, and startup height configuration. Includes `height: Arc<AtomicU64>` to share block-height tracking between the mailbox and the caller's event sink.
- `AppAdapter`: maps vendor `Application/VerifyingApplication/Reporter` callbacks to consensus traits. Internally tracks finalized blocks via `Arc<RwLock<HashMap<Digest, B>>>` shared across all clones. Reporter forwards finalization events to the caller-provided `EventSink`.
- `CommonwareEngine`: sealed constructor/startup for mailbox, adapter, and simplex engine. Uses the caller-provided `EventSink` for finalization side-effects (e.g. block persistence). Wires payload relay on `start()`.
- `Mailbox`: manages block proposals and relay broadcasting. `with_relay()` constructor activates block payload broadcasting via `Channel::PAYLOAD`.
- `PayloadRelayMessage`: wire format `[32-byte digest][encoded block bytes]` for relayed payloads.
- `payload_receive_loop`: async inbound handler that decodes, validates, and stores received block payloads.

## Clone Semantics
`AppAdapter::Clone` clones the `Arc` to the shared `finalized_blocks` map. All clones (batcher, voter/reporter) operate on the same block store. `remember_block()` is async (acquires write lock).
`Mailbox::Clone` clones `Option<BlockStore<B>>` (Arc-based) and `Option<mpsc::UnboundedSender<Bytes>>`.

## Data Flow
- propose: vendor -> `AppAdapter::propose` -> `ConsensusApp::propose` -> `remember_block()`
- verify: vendor -> `AppAdapter::verify` -> `ConsensusApp::verify` -> `remember_block()`
- genesis: vendor -> `AppAdapter::genesis` -> `ConsensusApp::genesis` -> `remember_block()`
- relay broadcast: `Mailbox::broadcast(digest)` -> `BlockStore` lookup -> `Codec::encode()` -> `PayloadRelayMessage::encode_wire()` -> mpsc -> forwarder task -> `NetworkSender::send(Recipients::All, wire, false)` on `Channel::PAYLOAD`
- relay receive: `payload_receive_loop` -> `NetworkReceiver::recv()` on `Channel::PAYLOAD` -> `PayloadRelayMessage::decode_wire()` -> validate digest -> `BlockStore::insert()`
- finalized event: vendor `Activity::Finalization` -> `AppAdapter::report` -> removes block from shared `finalized_blocks` -> `ConsensusEvent::Finalized` -> `EventSink::handle` -> ack

## Status
Complete. `CommonwareBlock` moved to `traits.rs`; imports use canonical `::traits::` paths. `finalized_blocks` uses `Arc<RwLock<HashMap>>` for cross-clone sharing. Payload relay activated via `Channel::PAYLOAD` (channel 3).
