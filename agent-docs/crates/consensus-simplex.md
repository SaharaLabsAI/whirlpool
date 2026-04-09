# consensus-simplex: Simplex BFT Adapter

## Summary
The `consensus-simplex` crate provides an adapter that bridges Whirlpool's abstract consensus traits to the Commonware Simplex BFT implementation, including a payload relay for broadcasting proposed blocks to peers over P2P.

Location: `crates/consensus/simplex/`

## Key Components

### CommonwareConfig
Holds parameters for the Simplex engine.
- `height: Arc<AtomicU64>`: Caller-owned shared height tracker used for recovery and block production (crates/consensus/simplex/src/config.rs:54).

### AppAdapter
Bridges `ConsensusApp` and `EventSink` to vendor traits.
- Implements `Application`, `VerifyingApplication`, and `Reporter` (crates/consensus/simplex/src/adapter.rs:89,124,153).
- Trait bounds for `Application`, `VerifyingApplication`, and `Reporter` do not require `S: Clone` as the sink is accessed via `Arc` (crates/consensus/simplex/src/adapter.rs:93,128,156).

### CommonwareEngine
The primary entry point for starting the consensus engine.
- Uses the caller-provided `EventSink` passed to `AppAdapter` for finalization events (crates/consensus/simplex/src/engine.rs).
- Shared `height` Arc is passed to `MailboxActor` to track the current chain tip (crates/consensus/simplex/src/engine.rs).
- `start()` wires the payload relay: creates an mpsc channel, constructs `Mailbox::with_relay()`, spawns an outbound forwarder task (reads from channel, sends via `NetworkSender::send(Recipients::All, ...)`), and spawns an inbound `payload_receive_loop` task.
- Trait bounds on block type: `CommonwareBlock + Encode + Decode<Cfg = ()>`.

### Mailbox
Manages block proposals and relay broadcasting (crates/consensus/simplex/src/mailbox.rs).
- `Mailbox::new(block_store)`: No relay — `broadcast()` is a no-op.
- `Mailbox::with_relay(block_store_for_relay, block_store_for_actor, payload_tx)`: Active relay — `broadcast(digest)` looks up the block by digest in `BlockStore`, encodes it via `Codec::encode()`, wraps in `PayloadRelayMessage`, and sends through the mpsc channel.
- `Mailbox` is `Clone`-safe: uses `Option<BlockStore<B>>` (Arc-based) and `Option<mpsc::UnboundedSender<Bytes>>`.

### PayloadRelayMessage
Wire format for relayed block payloads (crates/consensus/simplex/src/mailbox.rs).
- Format: `[32-byte SHA-256 digest][encoded block bytes]`
- `encode_wire(digest, block_bytes) -> Bytes`
- `decode_wire(buf) -> Result<(Digest, Bytes)>`

### payload_receive_loop
Inbound payload receiver (crates/consensus/simplex/src/receiver.rs).
- `payload_receive_loop<B, R>(receiver, block_store)`: Async loop that receives `NetworkMessage`s, decodes `PayloadRelayMessage`, validates digest by recomputing SHA-256 from decoded block, and stores in `BlockStore`.
- Generic over any `NetworkReceiver` where `R::PeerId: Debug`.
- Block type bound: `CommonwareBlock + Decode<Cfg = ()>`.

## Data Flow
1. **Proposal**: `CommonwareEngine` -> `MailboxActor` reads `height` -> `ConsensusApp::propose`.
2. **Relay Broadcast (outbound)**: `Mailbox::broadcast(digest)` -> lookup block in `BlockStore` -> encode -> `PayloadRelayMessage::encode_wire()` -> mpsc channel -> forwarder task -> `NetworkSender::send(Recipients::All, wire, false)` on `Channel::PAYLOAD`.
3. **Relay Receive (inbound)**: `payload_receive_loop` reads from `NetworkReceiver` on `Channel::PAYLOAD` -> `PayloadRelayMessage::decode_wire()` -> validate digest -> `BlockStore::insert()`.
4. **Finalization**: Vendor engine -> `AppAdapter::report` -> `EventSink::handle(Finalized)`.
5. **Persistence**: `PersistingFinalizationSink` (in `whirlpool-node`) receives event -> stores block -> increments shared `height` Arc.

## Module Layout
- `adapter.rs` — `AppAdapter` vendor trait bridge
- `config.rs` — `CommonwareConfig`
- `engine.rs` — `CommonwareEngine` with relay wiring
- `mailbox.rs` — `Mailbox`, `PayloadRelayMessage`, `BlockStore`
- `receiver.rs` — `payload_receive_loop` inbound handler
- `sink.rs` — Event sink utilities
- `traits.rs` — `CommonwareBlock` trait
