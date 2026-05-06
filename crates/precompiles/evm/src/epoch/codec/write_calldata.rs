use alloy_primitives::Bytes;
use alloy_sol_types::SolCall;

use crate::epoch::codec::dispatch::{
    advanceEpochCall, epochStartBlockCall, ADVANCE_EPOCH_SELECTOR,
};

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
