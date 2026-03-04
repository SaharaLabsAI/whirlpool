//! Commonware Simplex BFT adapter for consensus-core traits.

pub mod traits;
pub mod types;
pub mod config;
pub mod adapter;
pub mod engine;
pub mod mailbox;
pub mod sink;

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
