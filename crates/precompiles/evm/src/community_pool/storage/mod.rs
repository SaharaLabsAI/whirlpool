//! Community-pool storage slot and storage-value encoding helpers.

mod last_processed;
mod storage_encoding;
mod storage_slots;
mod value_slots;

pub use last_processed::community_pool_last_processed_epoch_slot;
pub use storage_encoding::{
    community_pool_last_processed_epoch_storage_slot, encode_u256_storage_value,
};
pub use storage_slots::{
    community_pool_locked_remaining_storage_slot,
    community_pool_unlock_amount_per_cycle_storage_slot,
    community_pool_unlock_every_epochs_storage_slot,
};
pub use value_slots::{
    community_pool_locked_remaining_slot, community_pool_unlock_amount_per_cycle_slot,
    community_pool_unlock_every_epochs_slot,
};
