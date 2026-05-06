use alloy_primitives::U256;

mod codec;
mod encoded_value;
mod epoch_start_mapping;
mod well_known_slots;
mod well_known_storage;

pub use codec::{decode_epoch_start_block_storage_value, decode_u64_storage_value};
pub use encoded_value::{
    encode_epoch_start_block_storage_value, encode_u64_storage_value,
    epoch_start_block_storage_slot,
};
pub use epoch_start_mapping::epoch_start_block_slot;
pub use well_known_slots::{current_epoch_slot, epoch_blocks_slot, next_epoch_block_slot};
pub use well_known_storage::{
    current_epoch_storage_slot, epoch_blocks_storage_slot, next_epoch_block_storage_slot,
};

pub const CURRENT_EPOCH_SLOT: U256 = U256::ZERO;
pub const EPOCH_BLOCKS_SLOT: U256 = U256::from_limbs([1, 0, 0, 0]);
pub const NEXT_EPOCH_BLOCK_SLOT: U256 = U256::from_limbs([2, 0, 0, 0]);
pub const EPOCH_START_BLOCK_MAPPING_SLOT: U256 = U256::from_limbs([3, 0, 0, 0]);

#[cfg(test)]
mod tests;
