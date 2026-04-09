//! Vendor-agnostic P2P networking abstractions for Whirlpool.
//!
//! This crate provides trait-based abstractions for peer-to-peer networking,
//! allowing the consensus layer to remain decoupled from specific networking
//! implementations (e.g., commonware, libp2p).
//!
//! # Architecture
//!
//! The crate is organized around three core traits:
//!
//! - [`NetworkSender`] - Send messages to peers
//! - [`NetworkReceiver`] - Receive messages from peers  
//! - [`NetworkProvider`] - Create and manage network connections
//!
//! These traits are parameterized by a [`PeerId`] type that identifies peers.
//!
//! # Channels
//!
//! Messages are multiplexed over logical channels identified by [`Channel`].
//! Pre-defined channels include:
//!
//! - [`Channel::VOTE`] - For vote messages
//! - [`Channel::CERTIFICATE`] - For certificate messages
//! - [`Channel::RESOLVER`] - For resolver messages
//! - [`Channel::PAYLOAD`] - For payload relay messages
//!
//! # Example
//!
//! ```rust,ignore
//! use p2p::{NetworkProvider, NetworkSender, Channel, Recipients};
//! use bytes::Bytes;
//!
//! async fn send_vote<P: NetworkProvider>(provider: P) -> Result<(), p2p::P2pError> {
//!     let (sender, mut receiver) = provider.start()?;
//!     
//!     let vote_data = Bytes::from("vote payload");
//!     sender.send(Channel::VOTE, vote_data, Recipients::All).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod errors;
pub mod traits;
pub mod types;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

// Re-export key types for convenience
pub use errors::P2pError;
pub use traits::{NetworkProvider, NetworkReceiver, NetworkSender, PeerId};
pub use types::{Channel, NetworkChannel, NetworkMessage, Recipients};
