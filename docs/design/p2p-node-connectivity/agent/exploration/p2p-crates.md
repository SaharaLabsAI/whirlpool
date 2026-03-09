# Exploration: P2P Crate Architecture

## p2p trait crate (crates/p2p)

### Traits (src/traits.rs)
- `PeerId`: Debug + Clone + Hash
- `NetworkSender`: async send(channel: Channel, data: Bytes, recipients: Recipients)
- `NetworkReceiver`: async recv() -> Option<NetworkMessage>
- `NetworkProvider`: start() -> Result<(Box<dyn NetworkSender>, Box<dyn NetworkReceiver>), P2pError>

### Types (src/types.rs)
- `Channel(u64)` — multiplexing key
- `NetworkMessage { channel, data, sender }` — tagged message
- `Recipients { One(PeerId), Many(Vec<PeerId>) }` — routing
- Channel constants: `VOTE`, `CERTIFICATE`, `RESOLVER`

### Errors (src/errors.rs)
- `P2pError` — channel saturation, send/receive failures, shutdown, invalid params

### Mock (src/mock.rs)
- In-memory test provider using tokio mpsc channels

## p2p-commonware crate (crates/p2p-commonware)

### Builder (src/provider.rs)
- `CommonwareNetworkProviderBuilder` — collects signer, namespace, listen_addr, dialable_addr, bootstrappers, max_message_size, initial_validators
- `.build(context)` -> `(CommonwareNetworkProvider, OracleHandle)`
- **Gap**: `initial_validators` stored but NEVER passed to oracle → discovery is blind
- **Gap**: `bootstrappers` field exists but never populated in any caller

### Transport (src/traits.rs)
- `CommonwareTransport`: start_per_channel(self) -> PerChannelNetwork, oracle(&self) -> &Oracle
- Used by consensus-simplex for dedicated vote/cert/resolver channels

### Sender (src/sender.rs)
- Maps `Recipients` enum to Commonware's `Recipients` type
- Clones sender for each send (Commonware needs &mut self)

### Receiver (src/receiver.rs)
- **CRITICAL BUG**: Hard-codes `Channel(0)` when wrapping messages → channel info lost
- Should extract real channel from the Commonware stream context

### Multiplexer (src/lib.rs)
- `MultiplexSender` / `MultiplexReceiver` — polls channels round-robin, tags with correct Channel
- This part works correctly — it's the per-channel receiver that's broken
