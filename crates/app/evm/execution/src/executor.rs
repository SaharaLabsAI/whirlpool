//! Compatibility re-exports for the previous executor-facing API.
//!
//! New reviewers should start from `ingress`, `codec`, `block_pipeline`, and
//! `post_handle` instead of this shim.

pub use crate::block_pipeline::{EvmApplication, ProposedEvmPayload, RecoveredTx};
pub use crate::codec::{
    build_header_from_evm_block, decode_evm_transaction, decode_evm_transactions,
};
pub use crate::traits::StateDb;
