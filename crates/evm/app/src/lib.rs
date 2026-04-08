pub mod config;
pub mod error;
pub mod executor;
pub mod traits;

pub use config::{
    WhirlpoolEvmConfig, DEFAULT_PROPOSER_FEE_RECIPIENT, VALIDATOR_FEE_RECIPIENTS_REGISTRY,
};
pub use error::EvmAppError;
pub use executor::{
    build_header_from_evm_block, decode_evm_transaction, decode_evm_transactions, EvmApplication,
    ProposedEvmPayload, RecoveredTx,
};
