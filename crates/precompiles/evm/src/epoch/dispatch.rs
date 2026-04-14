use alloy_primitives::Bytes;
use alloy_sol_types::{sol, SolCall};

use super::EpochPrecompileError;

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

pub fn current_epoch_calldata() -> Bytes {
    Bytes::from(currentEpochCall {}.abi_encode())
}

pub fn next_epoch_block_calldata() -> Bytes {
    Bytes::from(nextEpochBlockCall {}.abi_encode())
}

pub fn epoch_blocks_calldata() -> Bytes {
    Bytes::from(epochBlocksCall {}.abi_encode())
}

pub fn epoch_start_block_calldata(epoch: u64) -> Bytes {
    Bytes::from(epochStartBlockCall { epoch }.abi_encode())
}

pub fn advance_epoch_calldata() -> Bytes {
    Bytes::from(advanceEpochCall {}.abi_encode())
}

pub fn is_advance_epoch_calldata(data: &[u8]) -> bool {
    if !data.starts_with(&ADVANCE_EPOCH_SELECTOR) {
        return false;
    }

    advanceEpochCall::abi_decode_validate(data).is_ok()
}
