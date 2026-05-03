use alloy_primitives::U256;

use crate::community_pool::COMMUNITY_POOL_LAST_PROCESSED_EPOCH_SLOT;

pub fn community_pool_last_processed_epoch_slot() -> U256 {
    COMMUNITY_POOL_LAST_PROCESSED_EPOCH_SLOT
}
