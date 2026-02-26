# p2p-commonware Crate

## Purpose

Bridges vendor-agnostic `p2p` trait system to Commonware P2P implementation. Provides multiplexing adapters that route messages across multiple logical channels through unified Commonware sender/receiver abstractions.

# Core Types

### CommonwareNetworkProviderBuilder

Builder for constructing a discovery-backed network provider from high-level inputs.

- **Method**: `new(signer: C, namespace: impl Into<Vec<u8>>) -> Self` — Initialize builder with signer and namespace.
- **Method**: `listen_addr(addr: SocketAddr) -> Self` — Set local listen address.
- **Method**: `dialable_addr(addr: SocketAddr) -> Self` — Set address other peers can use to dial this node.
- **Method**: `bootstrappers(bootstrappers: Vec<Bootstrapper<C::PublicKey>>) -> Self` — Set initial bootstrapper nodes.
- **Method**: `max_message_size(size: u32) -> Self` — Set maximum allowed message size (default 1MB).
- **Method**: `channel_config(config: ChannelConfig) -> Self` — Set channel-specific configuration (e.g. backlog).
- **Method**: `build<Ctx>(self, context: Ctx) -> (CommonwareNetworkProvider<Ctx, C>, OracleHandle<C::PublicKey>)` — Construct the provider and a handle for oracle updates. Requires a context implementing `Spawner + Clock + Network + ...`.

### OracleHandle

Handle for updating the validator set after construction.

- **Method**: `update_validators(epoch: u64, validators: impl IntoIterator<Item = PK>)` — Updates the set of peers allowed to connect for a given epoch.

### MultiplexSender

### MultiplexSender

Routes outbound messages to correct per-channel sender.

- **Field**: `senders: Arc<HashMap<Channel, CommonwareSender<S>>>`
- **Method**: `new(senders: HashMap<Channel, CommonwareSender<S>>) -> Self`
- **Trait**: Implements `NetworkSender` with `send(channel, data, recipients) -> Result<(), P2pError>`
- **Note**: Generic over `S: commonware_p2p::Sender` with bounds `Clone + Send + Sync + 'static`

### MultiplexReceiver

Merges multiple per-channel receivers into single stream via round-robin polling.

- **Field**: `receivers: Vec<(Channel, CommonwareReceiver<R>)>`
- **Field**: `_handle: Option<commonware_runtime::Handle<()>>` — runtime handle for async task management; optional for testing
- **Method**: `new(receivers, handle: Handle<()>) -> Self` — production constructor, requires handle
- **Method**: `new_for_test(receivers) -> Self` — test-only constructor, no handle required
- **Trait**: Implements `NetworkReceiver` with `recv() -> Option<NetworkMessage<PeerId>>`
- **Behavior**: Polls all receivers in round-robin order, tags messages with correct `Channel`, returns `None` when all exhausted

### CommonwarePeerId

Wraps Commonware public key as PeerId.

- **Implements**: `PeerId` trait with `to_bytes()`, `Clone`, `Debug`, `Eq`, `Hash`
- **Storage**: `CommonwarePeerId(S::PublicKey)`

## Testing Infrastructure

### MockCwReceiver

Test double for `commonware_p2p::Receiver` backed by tokio mpsc channel.

- **Type**: `struct MockCwReceiver { rx: UnboundedReceiver<(PublicKey, Bytes)> }`
- **Constructor**: `MockCwReceiver::new() -> (UnboundedSender, MockCwReceiver)`
- **Trait**: Implements `commonware_p2p::Receiver` with async `recv() -> Result<(PublicKey, Bytes)>`
- **Shutdown**: Closing sender causes recv to return `BrokenPipe` error

### RED-Phase Tests

1. `test_multiplex_receiver_tags_channel` — Single receiver on VOTE channel tagged correctly
2. `test_multiplex_receiver_merges_channels` — Three receivers across three channels, all messages routed with correct tags
3. `test_multiplex_receiver_returns_none_on_shutdown` — Empty receivers with closed channels return None

## Dependencies

- `p2p`: Vendor-agnostic trait layer (internal crate)
- `commonware-p2p`: Commonware sender/receiver traits (vendor)
- `commonware-cryptography`: Ed25519 keys (vendor)
- `commonware-runtime`: Async runtime handle (vendor)
- `bytes`, `thiserror`: Utilities
- `tokio`: Async runtime (dev-dependency)

## Key Design Decision

`MultiplexReceiver._handle` is `Option<Handle<()>>` instead of `Handle<()>` to support testing without a live runtime. Tests use `new_for_test()` to construct without handle; production uses `new()` with handle for lifecycle management.
