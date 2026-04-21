use alloy_primitives::B256;

use super::{CURRENT_EPOCH_SLOT, EPOCH_BLOCKS_SLOT, NEXT_EPOCH_BLOCK_SLOT};

pub fn current_epoch_storage_slot() -> B256 {
    B256::from(CURRENT_EPOCH_SLOT.to_be_bytes::<32>())
}

pub fn epoch_blocks_storage_slot() -> B256 {
    B256::from(EPOCH_BLOCKS_SLOT.to_be_bytes::<32>())
}

pub fn next_epoch_block_storage_slot() -> B256 {
    B256::from(NEXT_EPOCH_BLOCK_SLOT.to_be_bytes::<32>())
}
