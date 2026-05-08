use alloy_primitives::{Bytes, U256};

use crate::community_pool::CommunityPoolPrecompileError;

pub fn decode_community_pool_balance_output(
    payload: &Bytes,
) -> Result<U256, CommunityPoolPrecompileError> {
    if payload.len() != 32 {
        return Err(CommunityPoolPrecompileError::InvalidReturnPayload);
    }

    let mut word = [0u8; 32];
    word.copy_from_slice(payload.as_ref());
    Ok(U256::from_be_bytes(word))
}

pub fn encode_u256_word(value: U256) -> Bytes {
    Bytes::copy_from_slice(&value.to_be_bytes::<32>())
}
