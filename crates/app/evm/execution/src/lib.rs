pub mod block_pipeline;
mod canonical_extra_data;
pub mod codec;
pub mod config;
pub mod error;
pub mod ingress;
pub mod post_handle;
pub mod traits;

pub use block_pipeline::{EvmApplication, ProposedEvmPayload};
pub use codec::{
    build_header_from_evm_block, decode_evm_transaction, decode_evm_transactions, RecoveredTx,
};
pub use config::{
    WhirlpoolEvmConfig, DEFAULT_PROPOSER_FEE_RECIPIENT, VALIDATOR_FEE_RECIPIENTS_REGISTRY,
};
pub use error::EvmAppError;
