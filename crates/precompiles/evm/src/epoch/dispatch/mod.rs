use alloy_sol_types::{sol, SolCall};

use super::EpochPrecompileError;

mod read_calldata;
mod write_calldata;

pub use read_calldata::{current_epoch_calldata, epoch_blocks_calldata, next_epoch_block_calldata};
pub use write_calldata::{
    advance_epoch_calldata, epoch_start_block_calldata, is_advance_epoch_calldata,
};

sol! {
    function currentEpoch() external view returns (uint64);
    function nextEpochBlock() external view returns (uint64);
    function epochBlocks() external view returns (uint64);
    function epochStartBlock(uint64 epoch) external view returns (uint64);
    function advanceEpoch() external;
}

pub const ADVANCE_EPOCH_SELECTOR: [u8; 4] = advanceEpochCall::SELECTOR;

pub enum EpochCall {
    CurrentEpoch,
    NextEpochBlock,
    EpochBlocks,
    EpochStartBlock { epoch: u64 },
    AdvanceEpoch,
}

pub fn decode_call(data: &[u8]) -> Result<EpochCall, EpochPrecompileError> {
    if data.len() < 4 {
        return Err(EpochPrecompileError::CalldataTooShort);
    }

    if data.starts_with(&currentEpochCall::SELECTOR) {
        currentEpochCall::abi_decode_validate(data)
            .map_err(|_| EpochPrecompileError::InvalidCurrentEpochCalldata)?;
        return Ok(EpochCall::CurrentEpoch);
    }

    if data.starts_with(&nextEpochBlockCall::SELECTOR) {
        nextEpochBlockCall::abi_decode_validate(data)
            .map_err(|_| EpochPrecompileError::InvalidNextEpochBlockCalldata)?;
        return Ok(EpochCall::NextEpochBlock);
    }

    if data.starts_with(&epochBlocksCall::SELECTOR) {
        epochBlocksCall::abi_decode_validate(data)
            .map_err(|_| EpochPrecompileError::InvalidEpochBlocksCalldata)?;
        return Ok(EpochCall::EpochBlocks);
    }

    if data.starts_with(&epochStartBlockCall::SELECTOR) {
        let call = epochStartBlockCall::abi_decode_validate(data)
            .map_err(|_| EpochPrecompileError::InvalidEpochStartBlockCalldata)?;
        return Ok(EpochCall::EpochStartBlock { epoch: call.epoch });
    }

    if data.starts_with(&advanceEpochCall::SELECTOR) {
        advanceEpochCall::abi_decode_validate(data)
            .map_err(|_| EpochPrecompileError::InvalidAdvanceEpochCalldata)?;
        return Ok(EpochCall::AdvanceEpoch);
    }

    Err(EpochPrecompileError::UnsupportedSelector)
}
