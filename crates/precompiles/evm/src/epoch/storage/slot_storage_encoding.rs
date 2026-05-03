use alloy_primitives::{B256, U256};

use crate::epoch::storage::epoch_start_block_slot;

pub fn epoch_start_block_storage_slot(epoch: u64) -> B256 {
    B256::from(epoch_start_block_slot(epoch).to_be_bytes::<32>())
}

pub fn encode_u64_storage_value(value: u64) -> B256 {
    B256::from(U256::from(value).to_be_bytes::<32>())
}

pub fn encode_epoch_start_block_storage_value(start_block: u64) -> B256 {
    let plus_one = start_block
        .checked_add(1)
        .expect("epoch start block + 1 must fit into u64");
    encode_u64_storage_value(plus_one)
}
