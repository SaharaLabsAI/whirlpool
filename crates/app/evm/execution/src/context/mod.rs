//! Runtime execution context inputs for the EVM application.
//!
//! `config/` owns static construction and EVM adapter composition. This module
//! owns node-local/runtime inputs that the block pipeline consumes while
//! proposing or verifying blocks. Runtime validator membership is still read at
//! the pipeline timing boundary in `block_pipeline::validators`, not here.

pub mod dkg;
pub mod proposer;
