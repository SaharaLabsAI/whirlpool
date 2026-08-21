# consensus-simplex: Simplex BFT Adapter

## Summary
The `consensus-simplex` crate provides an adapter that bridges Whirlpool's abstract consensus traits to the Commonware Simplex BFT implementation, including a payload relay for broadcasting proposed blocks to peers over P2P.

Location: `crates/consensus/simplex/`

## Key Components

### CommonwareConfig
Holds parameters for the Simplex engine.
- `height: Arc<AtomicU64>`: Caller-owned shared height tracker used for recovery/finalization progress and observability (crates/consensus/simplex/src/config.rs:54).
- `signing_scheme: SigningSchemeConfig`: runtime-selected consensus signature mode.
  - `Ed25519 { signer, validators }` keeps legacy behavior.
  - `BlsThresholdVrf { participants, polynomial, share }` enables threshold-BLS certificates while preserving ed25519 participant identities.

### AppAdapter
Bridges `ConsensusApp` and `EventSink` to vendor traits.
- Implements `Application` (propose + verify) and `Reporter` (crates/consensus/simplex/src/adapter.rs:89,124).
- Trait bounds for `Application` and `Reporter` do not require `S: Clone` as the sink is accessed via `Arc` (crates/consensus/simplex/src/adapter.rs:93,156).

### CommonwareEngine
The primary entry point for starting the consensus engine.
- Uses the caller-provided `EventSink` passed to `AppAdapter` for finalization events (crates/consensus/simplex/src/engine/mod.rs).
- Shared `height` Arc is still wired through engine setup, but mailbox proposal parent authority comes from Simplex `Context.parent.1`; height is status/progress, not parent selection (crates/consensus/simplex/src/engine/mod.rs, crates/consensus/simplex/src/mailbox/actor.rs).
- `start()` is an async fn that wires the payload relay: fetches the genesis block from the app, seeds the `BlockStore` and `Floor::Genesis` anchor, creates an mpsc channel, constructs `Mailbox::with_relay()`, spawns an outbound forwarder task (reads from channel, sends via `NetworkSender::send(Recipients::All, ...)`), and spawns an inbound `payload_receive_loop` task.
- Trait bounds on block type: `CommonwareBlock + Encode + Decode<Cfg = ()>`.
- `start()` now branches on `SigningSchemeConfig` and instantiates either:
  - `simplex::scheme::ed25519::Scheme::signer(...)`, or
  - `simplex::scheme::bls12381_threshold::vrf::Scheme::signer(...)`.
- Clippy hygiene: actor task exits without explicit unit-expression tails; digest validation compares directly against the all-`0xff` array.

### Mailbox
Manages block proposals and relay broadcasting (crates/consensus/simplex/src/mailbox/mod.rs).
- `Mailbox::new(sender)`: No relay — `broadcast()` is a no-op.
- `Mailbox::with_relay(sender, block_store, payload_tx)`: Active relay — `broadcast(digest)` looks up the block by digest in `BlockStore`, encodes it via `Codec::encode()`, wraps in `PayloadRelayMessage`, and sends through the mpsc channel.
- `Mailbox` forwards `Context<Digest, PublicKey>` into `MailboxActor` for `propose` and `verify`; proposals resolve `Context.parent.1` from genesis/cache and never fall back to the height counter.
- `BlockStore` is a payload availability cache. `MailboxActor` rechecks cached block digests before app verification, resolves only the Simplex context parent, and accepts a block link to that resolved parent by vendor parent digest or app parent id when those identities differ.
- Height-1 verification may use local genesis as the app parent only when the Simplex context parent resolves to this validator's local genesis; the EVM app verifier then accepts only parent ids that match an active validator genesis carrier. Later missing parents remain pending and cancellable. Digest mismatch, wrong context-parent linkage, or app rejection returns `false`.
- `Mailbox` is `Clone`-safe: uses `Option<BlockStore<B>>` (Arc-based) and `Option<mpsc::UnboundedSender<Bytes>>`.

### PayloadRelayMessage
Wire format for relayed block payloads (crates/consensus/simplex/src/mailbox/payload.rs).
- Format: `[32-byte SHA-256 digest][encoded block bytes]`
- `encode_wire(digest, block_bytes) -> Bytes`
- `decode_wire(buf) -> Result<(Digest, Bytes)>`

### payload_receive_loop
Inbound payload receiver (crates/consensus/simplex/src/receiver.rs).
- `payload_receive_loop<B, R>(receiver, block_store)`: Async loop that receives `NetworkMessage`s, decodes `PayloadRelayMessage`, validates digest by recomputing SHA-256 from decoded block, and stores in `BlockStore`.
- Generic over any `NetworkReceiver` where `R::PeerId: Debug`.
- Block type bound: `CommonwareBlock + Decode<Cfg = ()>`.

## Data Flow
1. **Proposal**: `CommonwareEngine` -> `Mailbox::propose(ctx)` -> `MailboxActor` resolves `ctx.parent.1` from genesis or `BlockStore` -> `ConsensusApp::propose(parent, parent.height + 1)` -> store proposed block by recomputed digest.
2. **Verification**: `CommonwareEngine` -> `Mailbox::verify(ctx, digest)` -> wait for cached block if temporarily missing -> recheck digest -> resolve only `ctx.parent.1` from genesis/cache -> require the block to link to that resolved parent, with a height-1 local-genesis compatibility path for EVM carrier ids -> `ConsensusApp::verify(parent, block)`. Height-1 EVM blocks can verify against local genesis only when the app accepts the carried parent id as an active-validator genesis carrier.
3. **Relay Broadcast (outbound)**: `Mailbox::broadcast(digest)` -> lookup block in `BlockStore` -> encode -> `PayloadRelayMessage::encode_wire()` -> mpsc channel -> forwarder task -> `NetworkSender::send(Recipients::All, wire, false)` on `Channel::PAYLOAD`.
4. **Relay Receive (inbound)**: `payload_receive_loop` reads from `NetworkReceiver` on `Channel::PAYLOAD` -> `PayloadRelayMessage::decode_wire()` -> validate digest -> `BlockStore::insert()`.
5. **Finalization**: Vendor engine -> `AppAdapter::report` -> `EventSink::handle(Finalized)`.
6. **Persistence**: `PersistingFinalizationSink` (in `whirlpool-node`) receives event -> stores block -> increments shared `height` Arc.

## Module Layout
- `adapter.rs` — `AppAdapter` vendor trait bridge
- `config.rs` — `CommonwareConfig`
- `engine/mod.rs` + `engine/tests/mod.rs` — `CommonwareEngine` with relay wiring and crate-local engine tests
- `mailbox/mod.rs` — `Mailbox` trait bridge + shared digest helpers
- `mailbox/actor.rs` — `MailboxActor`
- `mailbox/payload.rs` — `PayloadRelayMessage` wire envelope
- `mailbox/tests/mod.rs` — mailbox-focused tests, including parent-authority, pending payload/parent, app-abstain, cancellation, and concurrency-cap regressions
- `tests/mod.rs` — shared test fixtures (`TestBlock`, `MockApp`, `MockTxApp`) and crate-level tests
- `receiver.rs` — `payload_receive_loop` inbound handler
- `sink.rs` — Event sink utilities
- `traits.rs` — `CommonwareBlock` trait
