//! Core types for P2P networking.

use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// A channel identifier for multiplexing different message types over the network.
///
/// Channels allow logically separating different kinds of network traffic
/// (e.g., votes, certificates, resolver messages) while sharing the same
/// physical network connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Channel(pub u64);

impl Channel {
    /// Channel for vote messages.
    pub const VOTE: Channel = Channel(0);

    /// Channel for certificate messages.
    pub const CERTIFICATE: Channel = Channel(1);

    /// Channel for resolver messages.
    pub const RESOLVER: Channel = Channel(2);
}

/// Specifies the intended recipients of a network message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recipients<PeerId> {
    /// Send to all connected peers.
    All,

    /// Send to a specific single peer.
    One(PeerId),

    /// Send to multiple specific peers.
    Many(Vec<PeerId>),
}

/// A message channel configuration pairing a channel ID with its buffer capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkChannel {
    /// The channel identifier.
    pub channel: Channel,

    /// The maximum number of messages that can be buffered on this channel.
    pub capacity: usize,
}

impl NetworkChannel {
    /// Creates a new network channel configuration.
    pub fn new(channel: Channel, capacity: usize) -> Self {
        Self { channel, capacity }
    }
}

/// A received network message with metadata.
#[derive(Debug, Clone)]
pub struct NetworkMessage<PeerId> {
    /// The channel on which this message was received.
    pub channel: Channel,

    /// The message payload.
    pub data: Bytes,

    /// The peer that sent this message.
    pub peer_id: PeerId,
}

impl<PeerId> NetworkMessage<PeerId> {
    /// Creates a new network message.
    pub fn new(channel: Channel, data: Bytes, peer_id: PeerId) -> Self {
        Self {
            channel,
            data,
            peer_id,
        }
    }
}
