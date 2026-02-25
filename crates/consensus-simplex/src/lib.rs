//! Commonware Simplex BFT adapter for consensus-core traits.

pub mod types;
pub mod config;
pub mod adapter;
pub mod engine;
pub mod mailbox;
pub mod sink;

pub use types::CommonwareBlock;
pub use config::CommonwareConfig;
pub use adapter::AppAdapter;
pub use engine::CommonwareEngine;
pub use mailbox::{Mailbox, MailboxActor, Message};
pub use sink::FinalizationSink;

#[cfg(test)]
mod tests;
