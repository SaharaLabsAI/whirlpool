//! Commonware Simplex BFT adapter for consensus-core traits.

use commonware_cryptography::sha256::Digest;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod adapter;
pub mod config;
pub mod engine;
pub mod mailbox;
pub mod receiver;
pub mod sink;
pub mod traits;
pub mod types;

/// Shared block store keyed by digest.
///
/// Both [`MailboxActor`] (producer) and [`AppAdapter`] (reporter) hold a
/// handle to the same store so finalization can find blocks that were
/// created during propose/genesis.
pub type BlockStore<B> = Arc<RwLock<HashMap<Digest, B>>>;

pub use adapter::AppAdapter;
pub use config::{CommonwareConfig, SigningSchemeConfig};
pub use engine::CommonwareEngine;
pub use mailbox::{Mailbox, MailboxActor, Message, PayloadRelayMessage};
pub use receiver::payload_receive_loop;
pub use sink::FinalizationSink;

// Channel constants for P2P communication
pub use network::Channel;
pub const VOTE_CHANNEL: Channel = Channel(0);
pub const CERTIFICATE_CHANNEL: Channel = Channel(1);
pub const RESOLVER_CHANNEL: Channel = Channel(2);
#[cfg(test)]
mod tests;
