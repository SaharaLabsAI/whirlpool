use alloy_primitives::B256;

use super::{
    COMMUNITY_POOL_LOCKED_REMAINING_SLOT, COMMUNITY_POOL_UNLOCK_AMOUNT_PER_CYCLE_SLOT,
    COMMUNITY_POOL_UNLOCK_EVERY_EPOCHS_SLOT,
};

pub fn community_pool_unlock_every_epochs_storage_slot() -> B256 {
    B256::from(COMMUNITY_POOL_UNLOCK_EVERY_EPOCHS_SLOT.to_be_bytes::<32>())
}

pub fn community_pool_unlock_amount_per_cycle_storage_slot() -> B256 {
    B256::from(COMMUNITY_POOL_UNLOCK_AMOUNT_PER_CYCLE_SLOT.to_be_bytes::<32>())
}

pub fn community_pool_locked_remaining_storage_slot() -> B256 {
    B256::from(COMMUNITY_POOL_LOCKED_REMAINING_SLOT.to_be_bytes::<32>())
}
