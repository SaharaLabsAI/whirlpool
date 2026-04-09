//! Vendor-agnostic trait abstractions for P2P networking.

use crate::{
    errors::P2pError,
    types::{Channel, NetworkMessage, Recipients},
};
use bytes::Bytes;
use std::fmt::Debug;

/// A peer identifier in the network.
///
/// This trait abstracts over different peer identification schemes.
/// Implementations must be cloneable (but not necessarily `Copy`) to
/// accommodate public key types like `ed25519::PublicKey`.
pub trait PeerId: Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static {}

/// Capability to send messages to the network.
///
/// Implementations of this trait handle outbound network communication,
/// routing messages to the appropriate recipients on the specified channel.
#[allow(async_fn_in_trait)]
pub trait NetworkSender: Send + Sync + 'static {
    /// The peer identifier type used by this network implementation.
    type PeerId: PeerId;

    /// Sends a message on the specified channel to the given recipients.
    ///
    /// # Arguments
    ///
    /// * `channel` - The logical channel to send on
    /// * `data` - The message payload (uses `Bytes` for efficient zero-copy passing)
    /// * `recipients` - Who should receive this message
    ///
    /// # Errors
    ///
    /// Returns `P2pError::ChannelFull` if the send buffer is full.
    /// Returns `P2pError::SendFailed` if the message cannot be delivered.
    async fn send(
        &self,
        channel: Channel,
        data: Bytes,
        recipients: Recipients<Self::PeerId>,
    ) -> Result<(), P2pError>;
}

/// Capability to receive messages from the network.
///
/// Implementations of this trait provide access to inbound network messages
/// from all configured channels.
#[allow(async_fn_in_trait)]
pub trait NetworkReceiver: Send + 'static {
    /// The peer identifier type used by this network implementation.
    type PeerId: PeerId;

    /// Receives the next message from any channel.
    ///
    /// Returns `None` when the network has been shut down and no more
    /// messages will be delivered.
    ///
    /// # Cancel Safety
    ///
    /// This method should be cancel-safe: dropping the future should not
    /// lose messages.
    async fn recv(&mut self) -> Option<NetworkMessage<Self::PeerId>>;
}

/// Provider that creates and manages network connections.
///
/// Implementations of this trait are responsible for establishing network
/// connectivity, managing peer connections, and creating the sender/receiver
/// halves for application use.
pub trait NetworkProvider {
    /// The peer identifier type used by this network implementation.
    type PeerId: PeerId;

    /// The sender type for outbound messages.
    type Sender: NetworkSender<PeerId = Self::PeerId>;

    /// The receiver type for inbound messages.
    type Receiver: NetworkReceiver<PeerId = Self::PeerId>;

    /// Starts the network provider and returns sender/receiver handles.
    ///
    /// This method initializes the network stack, establishes connections,
    /// and returns handles for sending and receiving messages.
    ///
    /// # Returns
    ///
    /// A tuple of `(sender, receiver)` that can be used to communicate
    /// over the network.
    ///
    /// # Errors
    ///
    /// Returns `P2pError` if the network cannot be initialized.
    fn start(self) -> Result<(Self::Sender, Self::Receiver), P2pError>;
}
