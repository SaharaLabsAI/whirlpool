use alloy_primitives::Bytes;
use alloy_sol_types::SolCall;

use crate::epoch::dispatch::{currentEpochCall, epochBlocksCall, nextEpochBlockCall};

pub fn current_epoch_calldata() -> Bytes {
    Bytes::from(currentEpochCall {}.abi_encode())
}

pub fn next_epoch_block_calldata() -> Bytes {
    Bytes::from(nextEpochBlockCall {}.abi_encode())
}

pub fn epoch_blocks_calldata() -> Bytes {
    Bytes::from(epochBlocksCall {}.abi_encode())
}
