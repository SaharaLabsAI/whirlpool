# `core` — network interfaces (shape)

**Purpose**: component *interfaces* expressed as Rust traits.

Owns: the boundary between consensus and the concrete network implementation.

Why: the chain should be able to swap consensus engines without rewriting the network boundary; consensus should depend only on generic message transport.

Inputs/outputs: authenticated `PeerId` plus raw message bytes.

Depends on: `types` (and minimal `async`/error plumbing if needed).

Not in scope: discovery, peer scoring, wire formats, encryption details, concrete transports.

See also:

- [`network crate`](../network.md) (concrete implementation responsibilities)
- Simplex-specific channel mapping lives under consensus:
  - [`simplex/network/channels`](../consensus/engines/simplex/network/channels.md)
  - [`simplex/network/marshal_mailbox`](../consensus/engines/simplex/network/marshal_mailbox/index.md)

```rust
// NOTE: pseudocode for boundaries (not a spec; syntax intentionally loose).
//
// Design rule: `core` depends only on `types`.
// Do NOT hard-depend on a concrete P2P crate from `core`.
//
// In practice, these traits are satisfied by Commonware's P2P traits
// (e.g. `commonware_p2p::{Sender, Receiver}`) and `bytes::Bytes`, but `core`
// describes the *shape* only.

/// Authenticated peer identity (typically a validator public key).
pub type PeerId = types::PublicKey;

/// Opaque identifier for a logical message channel.
///
/// Channel IDs are *assigned by the consumer* (e.g. a consensus engine, a payload
/// distribution engine, etc.). The network does not interpret the meaning.
pub struct ChannelId(pub u16);

/// Implementation-defined tuning knobs for a channel.
///
/// This is intentionally generic: the network may implement rate limits, buffering,
/// backpressure, prioritization, and/or per-peer quotas behind the scenes.
pub struct ChannelConfig {
  pub quota: usize,
  pub backlog: usize,
  // Optional examples (intentionally not exhaustive):
  // pub max_message_bytes: usize,
  // pub priority: u8,
}

/// A minimal sender over an authenticated peer identity.
pub trait ChannelSender {
  type PeerId;
  type Error;

  async fn send(&self, to: Self::PeerId, bytes: Vec<u8>) -> Result<(), Self::Error>;
}

/// A minimal receiver over an authenticated peer identity.
pub trait ChannelReceiver {
  type PeerId;
  type Error;

  /// Returns (from, bytes).
  async fn recv(&mut self) -> Result<(Self::PeerId, Vec<u8>), Self::Error>;
}

/// Consensus-agnostic channel boundary.
///
/// Consumers request one or more *logical channels* by registering `ChannelId`s.
/// A consensus engine is responsible for choosing the IDs and mapping them to its
/// own internal message classes.
pub trait ChannelNetwork {
  type PeerId;

  type Sender: ChannelSender<PeerId = Self::PeerId>;
  type Receiver: ChannelReceiver<PeerId = Self::PeerId>;

  /// Register/open a logical channel.
  ///
  /// Implementations may:
  /// - multiplex multiple channels over one transport
  /// - enforce quotas/backpressure
  /// - drop/ban peers for abuse (outside this trait)
  fn register_channel(
    &mut self,
    channel: ChannelId,
    cfg: ChannelConfig,
  ) -> (Self::Sender, Self::Receiver);
}

/// Network boundary.
///
/// `ChannelNetwork` is the minimum required to satisfy Commonware-style engines
/// that take `(Sender, Receiver)` pairs (e.g. Simplex).
///
/// Implementations may choose to expose additional capabilities (peer events,
/// membership management, blocking, etc.) via extra traits.
pub trait Network: ChannelNetwork {}

// Optional extensions (still consensus-agnostic):
//
// - `Blocker`: block/ban a peer from sending/receiving
// - `PeerSetManager`: view/update the current allowed peer set (membership)
//
// These are often required by backfill/resolver engines but are not tied to any
// particular consensus algorithm.
```

## Notes: engine-specific channel mappings

This file deliberately does **not** define concepts like "vote plane" or "certificate plane".
Those are engine-specific and belong in the engine’s docs.

For example, Commonware Simplex requires three channels (votes/certificates/resolver), and marshal mailbox introduces additional channels for payload distribution/backfill. Those mappings live under:

- [`simplex/network/channels`](../consensus/engines/simplex/network/channels.md)
- [`simplex/network/marshal_mailbox`](../consensus/engines/simplex/network/marshal_mailbox/index.md)

## Example mapping (Alto)

Alto registers one authenticated network and splits it into fixed channel IDs.

For Simplex consensus channels (engine-internal):

- `PENDING_CHANNEL = 0`   (votes)
- `RECOVERED_CHANNEL = 1` (certificates)
- `RESOLVER_CHANNEL = 2`  (consensus resolver)

Payload/marshal channels are intentionally documented under marshal mailbox, since they are not
part of the consensus-agnostic `core` boundary.

See:

- `vendor/alto/chain/src/bin/validator.rs`
- [`simplex/network/marshal_mailbox/channels`](../consensus/engines/simplex/network/marshal_mailbox/channels.md)
