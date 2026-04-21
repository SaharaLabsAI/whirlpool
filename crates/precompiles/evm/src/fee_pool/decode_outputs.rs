use alloy_primitives::{Bytes, U256};

use super::FeePoolPrecompileError;

pub fn decode_fee_pool_balance_output(payload: &Bytes) -> Result<U256, FeePoolPrecompileError> {
    decode_u256_output(payload)
}

pub fn decode_claimable_balance_output(payload: &Bytes) -> Result<U256, FeePoolPrecompileError> {
    decode_u256_output(payload)
}

pub fn decode_withdraw_output(payload: &Bytes) -> Result<U256, FeePoolPrecompileError> {
    decode_u256_output(payload)
}

fn decode_u256_output(payload: &Bytes) -> Result<U256, FeePoolPrecompileError> {
    if payload.len() != 32 {
        return Err(FeePoolPrecompileError::InvalidReturnPayload);
    }

    let mut word = [0u8; 32];
    word.copy_from_slice(payload.as_ref());
    Ok(U256::from_be_bytes(word))
}
