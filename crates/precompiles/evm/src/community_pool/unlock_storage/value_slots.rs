use alloy_primitives::U256;

use crate::community_pool::{
    COMMUNITY_POOL_LOCKED_REMAINING_SLOT, COMMUNITY_POOL_UNLOCK_AMOUNT_PER_CYCLE_SLOT,
    COMMUNITY_POOL_UNLOCK_EVERY_EPOCHS_SLOT,
};

pub fn community_pool_unlock_every_epochs_slot() -> U256 {
    COMMUNITY_POOL_UNLOCK_EVERY_EPOCHS_SLOT
}

pub fn community_pool_unlock_amount_per_cycle_slot() -> U256 {
    COMMUNITY_POOL_UNLOCK_AMOUNT_PER_CYCLE_SLOT
}

pub fn community_pool_locked_remaining_slot() -> U256 {
    COMMUNITY_POOL_LOCKED_REMAINING_SLOT
}
