use alloy_primitives::Bytes;

use crate::epoch::{decode_primary::decode_u64_output, EpochPrecompileError};

pub fn decode_epoch_blocks_output(payload: &Bytes) -> Result<u64, EpochPrecompileError> {
    decode_u64_output(payload)
}

pub fn decode_epoch_start_block_output(payload: &Bytes) -> Result<u64, EpochPrecompileError> {
    decode_u64_output(payload)
}
