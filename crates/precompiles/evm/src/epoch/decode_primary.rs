use alloy_primitives::{Bytes, U256};

use super::EpochPrecompileError;

pub fn decode_u64_output(payload: &Bytes) -> Result<u64, EpochPrecompileError> {
    if payload.len() != 32 {
        return Err(EpochPrecompileError::InvalidReturnPayload);
    }

    let mut word = [0u8; 32];
    word.copy_from_slice(payload.as_ref());
    let value = U256::from_be_bytes(word);
    u64::try_from(value).map_err(|_| EpochPrecompileError::ValueOutOfRange)
}

pub fn decode_current_epoch_output(payload: &Bytes) -> Result<u64, EpochPrecompileError> {
    decode_u64_output(payload)
}

pub fn decode_next_epoch_block_output(payload: &Bytes) -> Result<u64, EpochPrecompileError> {
    decode_u64_output(payload)
}
