//! Canonical trait boundary for the consensus crate.
//!
//! This module provides stable trait import paths while preserving
//! compatibility with existing module-level imports.

pub use crate::app::ConsensusApp;
pub use crate::block::Block;
pub use crate::engine::ConsensusEngine;
pub use crate::event::EventSink;
