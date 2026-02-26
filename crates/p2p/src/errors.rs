//! Error types for P2P networking operations.

use thiserror::Error;

/// Errors that can occur during P2P networking operations.
#[derive(Debug, Error, Clone)]
pub enum P2pError {
    /// The channel buffer is full and cannot accept more messages.
    #[error("channel full: unable to send message")]
    ChannelFull,

    /// Failed to send a message to the network.
    #[error("send failed: {0}")]
    SendFailed(String),

    /// Failed to receive a message from the network.
    #[error("receive failed: {0}")]
    ReceiveFailed(String),

    /// The network provider has been shut down.
    #[error("network shutdown")]
    NetworkShutdown,

    /// Invalid channel identifier.
    #[error("invalid channel: {0}")]
    InvalidChannel(u64),

    /// Invalid recipient specification.
    #[error("invalid recipients: {0}")]
    InvalidRecipients(String),
}
