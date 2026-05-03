use alloy_primitives::{B256, U256};

use crate::community_pool::COMMUNITY_POOL_LAST_PROCESSED_EPOCH_SLOT;

pub fn community_pool_last_processed_epoch_storage_slot() -> B256 {
    B256::from(COMMUNITY_POOL_LAST_PROCESSED_EPOCH_SLOT.to_be_bytes::<32>())
}

pub fn encode_u256_storage_value(value: U256) -> B256 {
    B256::from(value.to_be_bytes::<32>())
}
