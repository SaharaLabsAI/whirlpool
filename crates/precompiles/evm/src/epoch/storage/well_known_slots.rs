use alloy_primitives::U256;

use crate::epoch::storage::{CURRENT_EPOCH_SLOT, EPOCH_BLOCKS_SLOT, NEXT_EPOCH_BLOCK_SLOT};

pub fn current_epoch_slot() -> U256 {
    CURRENT_EPOCH_SLOT
}

pub fn epoch_blocks_slot() -> U256 {
    EPOCH_BLOCKS_SLOT
}

pub fn next_epoch_block_slot() -> U256 {
    NEXT_EPOCH_BLOCK_SLOT
}
