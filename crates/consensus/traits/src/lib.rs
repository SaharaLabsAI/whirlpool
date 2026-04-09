// consensus-core

pub mod block;

pub mod error;
pub use error::ConsensusError;

pub mod app;

pub mod event;
pub use event::ConsensusEvent;

pub mod engine;
pub use engine::{ConsensusStatus, RunningEngine};

pub mod traits;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

#[cfg(test)]
mod tests;
