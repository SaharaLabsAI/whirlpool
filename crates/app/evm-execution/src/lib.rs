pub mod block_pipeline;
pub mod codec;
pub mod config;
pub mod context;
pub mod error;
pub mod ingress;
mod invariants;
pub mod post_handle;
pub mod traits;

pub use block_pipeline::{EvmApplication, ProposedEvmPayload};
pub use codec::{
    build_header_from_evm_block, decode_evm_transaction, decode_evm_transactions, RecoveredTx,
};
pub use config::WhirlpoolEvmConfig;
pub use error::EvmAppError;
