mod canonical_extra_data;
pub mod config;
mod epoch_boundary;
pub mod error;
pub mod executor;
pub mod traits;
mod validator_activation;

pub use config::{
    WhirlpoolEvmConfig, DEFAULT_PROPOSER_FEE_RECIPIENT, VALIDATOR_FEE_RECIPIENTS_REGISTRY,
};
pub use epoch_boundary::EpochBoundaryHook;
pub use error::EvmAppError;
pub use executor::{
    build_header_from_evm_block, decode_evm_transaction, decode_evm_transactions, EvmApplication,
    ProposedEvmPayload, RecoveredTx,
};
