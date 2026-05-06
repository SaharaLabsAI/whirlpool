use alloy_primitives::{address, Address, Bytes, B256, U256};
use reth_evm::revm::precompile::PrecompileResult;

mod codec;
pub mod gas;
mod registration;
mod runtime;
pub mod storage;
mod transition;

pub use codec::{
    advance_epoch_calldata, current_epoch_calldata, decode_current_epoch_output,
    decode_epoch_blocks_output, decode_epoch_start_block_output, decode_next_epoch_block_output,
    decode_u64_output, epoch_blocks_calldata, epoch_start_block_calldata,
    is_advance_epoch_calldata, next_epoch_block_calldata,
};
pub use registration::{epoch_system_tx_sender, register};
pub use runtime::boundary_adapter::{
    apply_epoch_boundary_effect, execute_epoch_boundary_system_call_if_required,
    load_epoch_boundary_state, EpochBoundaryRuntimeError,
};
pub use storage::{
    current_epoch_slot, current_epoch_storage_slot, decode_epoch_start_block_storage_value,
    decode_u64_storage_value, encode_epoch_start_block_storage_value, encode_u64_storage_value,
    epoch_blocks_slot, epoch_blocks_storage_slot, epoch_start_block_slot,
    epoch_start_block_storage_slot, next_epoch_block_slot, next_epoch_block_storage_slot,
};
pub use transition::{
    boundary_required_for_height, extract_epoch_boundary_effect,
    reserved_advance_epoch_call_matches, EpochBoundaryEffect, EpochBoundaryEffectError,
    EpochBoundaryState, EpochBoundaryStorageWrite,
};

pub const EPOCH_PRECOMPILE_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000103");

pub const EPOCH_BLOCKS_DEFAULT: u64 = 403_200;
pub const EPOCH_SYSTEM_TX_GAS_LIMIT: u64 = 120_000;
pub const EPOCH_SYSTEM_TX_PRIVATE_KEY: B256 = B256::repeat_byte(0x42);
pub const EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI: u128 = 1_000_000_000_000_000_000;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EpochPrecompileError {
    #[error("calldata is too short")]
    CalldataTooShort,
    #[error("unsupported epoch selector")]
    UnsupportedSelector,
    #[error("invalid currentEpoch calldata")]
    InvalidCurrentEpochCalldata,
    #[error("invalid nextEpochBlock calldata")]
    InvalidNextEpochBlockCalldata,
    #[error("invalid epochBlocks calldata")]
    InvalidEpochBlocksCalldata,
    #[error("invalid epochStartBlock calldata")]
    InvalidEpochStartBlockCalldata,
    #[error("invalid advanceEpoch calldata")]
    InvalidAdvanceEpochCalldata,
    #[error("advanceEpoch cannot run in a static context")]
    StaticCallAdvanceEpoch,
    #[error("advanceEpoch caller is not authorized")]
    UnauthorizedAdvanceEpochCaller,
    #[error("advanceEpoch called at block {got} but expected boundary block {expected}")]
    InvalidBoundaryBlock { expected: u64, got: u64 },
    #[error("epoch {0} is not initialized")]
    EpochNotInitialized(u64),
    #[error("epoch start for epoch {0} is already initialized")]
    EpochStartAlreadyInitialized(u64),
    #[error("epoch storage value does not fit into uint64")]
    ValueOutOfRange,
    #[error("epoch arithmetic overflow")]
    ArithmeticOverflow,
    #[error("invalid epoch return payload")]
    InvalidReturnPayload,
}

fn encode_u64_word(value: u64) -> Bytes {
    Bytes::copy_from_slice(&U256::from(value).to_be_bytes::<32>())
}

fn encode_revert_reason(reason: &str) -> Bytes {
    let reason_bytes = reason.as_bytes();
    let padded_len = reason_bytes.len().div_ceil(32) * 32;
    let mut payload = Vec::with_capacity(4 + 32 * 3 + padded_len);
    payload.extend_from_slice(&[0x08, 0xc3, 0x79, 0xa0]);
    payload.extend_from_slice(&U256::from(32_u64).to_be_bytes::<32>());
    payload.extend_from_slice(&U256::from(reason_bytes.len()).to_be_bytes::<32>());
    payload.extend_from_slice(reason_bytes);
    payload.resize(4 + 32 * 2 + padded_len, 0);
    Bytes::from(payload)
}

fn revert_result(gas_used: u64, error: EpochPrecompileError) -> PrecompileResult {
    Ok(reth_evm::revm::precompile::PrecompileOutput::new_reverted(
        gas_used,
        encode_revert_reason(&error.to_string()),
    ))
}

#[cfg(test)]
mod tests;
