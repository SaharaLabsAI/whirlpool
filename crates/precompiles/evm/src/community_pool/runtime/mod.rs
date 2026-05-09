//! Runtime adapters that apply community-pool accounting effects to EVM state.

mod post_block_accounting;

pub use post_block_accounting::{apply_post_block_accounting, PostBlockAccountingRuntimeError};
