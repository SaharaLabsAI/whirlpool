# Simplex Adapter Bridge

The adapter crate translates Commonware Simplex APIs into Whirlpool consensus traits, and provides a payload relay for broadcasting proposed blocks to peers.

## Interface/Implementation Split
- Interface module: `crates/consensus/simplex/src/traits.rs`
  - `CommonwareBlock`
- Implementation modules:
  - `crates/consensus/simplex/src/adapter.rs`
  - `crates/consensus/simplex/src/engine/mod.rs`
  - `crates/consensus/simplex/src/mailbox/mod.rs`
  - `crates/consensus/simplex/src/mailbox/actor.rs`
  - `crates/consensus/simplex/src/mailbox/payload.rs`
  - `crates/consensus/simplex/src/receiver.rs`
  - `crates/consensus/simplex/src/sink.rs`
  - `crates/consensus/simplex/src/config.rs`

## Canonical Imports
- Core traits: `consensus::traits::{Block, ConsensusApp, EventSink, ConsensusEngine}`
- Adapter trait: `consensus_simplex::traits::CommonwareBlock`
- Relay types: `consensus_simplex::{PayloadRelayMessage, payload_receive_loop}`

## Core Types
- `CommonwareBlock`: super-trait requiring `consensus::traits::Block + commonware_consensus::Block + Clone`.
- `CommonwareConfig`: simplex timing, buffer, and startup height configuration. Includes `height: Arc<AtomicU64>` for progress tracking; mailbox proposal/verification parent identity is taken from Simplex `Context.parent.1`, not from height.
- `AppAdapter`: maps vendor `Application/VerifyingApplication/Reporter` callbacks to consensus traits. Internally tracks finalized blocks via `Arc<RwLock<HashMap<Digest, B>>>` shared across all clones. Reporter forwards finalization events to the caller-provided `EventSink`.
- `CommonwareEngine`: sealed constructor/startup for mailbox, adapter, and simplex engine. Uses the caller-provided `EventSink` for finalization side-effects (e.g. block persistence). Wires payload relay on `start()`.
- `Mailbox`: manages block proposals, verification requests, and relay broadcasting. `propose(ctx)`/`verify(ctx, digest)` preserve Simplex context for `MailboxActor`; `with_relay()` activates block payload broadcasting via `Channel::PAYLOAD`.
- `PayloadRelayMessage`: wire format `[32-byte digest][encoded block bytes]` for relayed payloads.
- `payload_receive_loop`: async inbound handler that decodes, validates, and stores received block payloads.

## Commonware 2026.4.0 Compatibility Notes
- Adapter propose/verify callbacks now use `commonware_consensus::marshal::ancestry::AncestorStream` with generic `BlockProvider` type parameters.
- `Mailbox` relay now implements plan-aware simplex relay (`broadcast(payload, simplex::Plan<_>)`).
- Engine wiring now uses `simplex::Config::page_cache` via `commonware_runtime::buffer::paged::CacheRef::from_pooler`, and maps timeout fields to `certification_timeout`/`timeout_retry` with `ForwardingPolicy::Disabled`.

## Clone Semantics
`AppAdapter::Clone` clones the `Arc` to the shared `finalized_blocks` map. All clones (batcher, voter/reporter) operate on the same block store. `remember_block()` is async (acquires write lock).
`Mailbox::Clone` clones `Option<BlockStore<B>>` (Arc-based) and `Option<mpsc::UnboundedSender<Bytes>>`.

## Data Flow
- adapter propose: vendor marshal ancestry -> `AppAdapter::propose` -> `ConsensusApp::propose` -> `remember_block()`
- adapter verify: vendor marshal ancestry -> `AppAdapter::verify` -> `ConsensusApp::verify` -> `remember_block()`
- mailbox propose: `Mailbox::propose(ctx)` -> actor resolves `ctx.parent.1` from genesis/cache -> `ConsensusApp::propose(parent, parent.height + 1)` -> store digest
- mailbox verify: `Mailbox::verify(ctx, digest)` -> actor waits for cached block if missing -> digest check -> resolve only `ctx.parent.1` from genesis/cache -> require block linkage to that resolved parent, with a height-1 local-genesis compatibility path for EVM carrier ids -> `ConsensusApp::verify`; height-1 EVM blocks may use local genesis only when app verification accepts the carried active-validator genesis parent id
- genesis: vendor -> `AppAdapter::genesis` or `MailboxActor` -> `ConsensusApp::genesis` -> `remember_block()`
- relay broadcast: `Mailbox::broadcast(digest)` -> `BlockStore` lookup -> `Codec::encode()` -> `PayloadRelayMessage::encode_wire()` -> mpsc -> forwarder task -> `NetworkSender::send(Recipients::All, wire, false)` on `Channel::PAYLOAD`
- relay receive: `payload_receive_loop` -> `NetworkReceiver::recv()` on `Channel::PAYLOAD` -> `PayloadRelayMessage::decode_wire()` -> validate digest -> `BlockStore::insert()`
- finalized event: vendor `Activity::Finalization` -> `AppAdapter::report` -> looks up and clones the block from shared `finalized_blocks` while retaining the cache for later parent resolution -> `ConsensusEvent::Finalized` -> `EventSink::handle` -> ack

## Status
Complete. `CommonwareBlock` moved to `traits.rs`; imports use canonical `::traits::` paths. `finalized_blocks` uses `Arc<RwLock<HashMap>>` for cross-clone sharing. Payload relay activated via `Channel::PAYLOAD` (channel 3). Mailbox proposals use `Context.parent.1`; verification treats `BlockStore` as a checked availability cache, resolves only the Simplex context parent, and can supply cached app parents when app parent IDs differ from consensus payload digests. Finalized blocks remain cached for later parent resolution.
