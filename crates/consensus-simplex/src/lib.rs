//! Commonware Simplex BFT adapter for consensus-core traits.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use commonware_cryptography::sha256::Digest;

pub mod traits;
pub mod types;
pub mod config;
pub mod adapter;
pub mod engine;
pub mod mailbox;
pub mod sink;

/// Shared block store keyed by digest.
///
/// Both [`MailboxActor`] (producer) and [`AppAdapter`] (reporter) hold a
/// handle to the same store so finalization can find blocks that were
/// created during propose/genesis.
pub type BlockStore<B> = Arc<RwLock<HashMap<Digest, B>>>;

pub use config::CommonwareConfig;
pub use adapter::AppAdapter;
pub use engine::CommonwareEngine;
pub use mailbox::{Mailbox, MailboxActor, Message};
pub use sink::FinalizationSink;

// Channel constants for P2P communication
pub use p2p::Channel;
pub const VOTE_CHANNEL: Channel = Channel(0);
pub const CERTIFICATE_CHANNEL: Channel = Channel(1);
pub const RESOLVER_CHANNEL: Channel = Channel(2);
#[cfg(test)]
mod tests;
