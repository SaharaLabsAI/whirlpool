//! Commonware Simplex BFT adapter for consensus-core traits.

pub mod types;
pub mod config;
pub mod adapter;
pub mod engine;

pub use types::CommonwareBlock;
pub use config::CommonwareConfig;
pub use adapter::AppAdapter;
pub use engine::CommonwareEngine;

#[cfg(test)]
mod tests;
