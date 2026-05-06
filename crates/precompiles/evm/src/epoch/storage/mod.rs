use alloy_primitives::U256;

mod decode;
mod slot_mapping;
mod slot_storage_encoding;
mod slot_storage_primary;
mod slot_value_primary;

pub use decode::{decode_epoch_start_block_storage_value, decode_u64_storage_value};
pub use slot_mapping::epoch_start_block_slot;
pub use slot_storage_encoding::{
    encode_epoch_start_block_storage_value, encode_u64_storage_value,
    epoch_start_block_storage_slot,
};
pub use slot_storage_primary::{
    current_epoch_storage_slot, epoch_blocks_storage_slot, next_epoch_block_storage_slot,
};
pub use slot_value_primary::{current_epoch_slot, epoch_blocks_slot, next_epoch_block_slot};

pub const CURRENT_EPOCH_SLOT: U256 = U256::ZERO;
pub const EPOCH_BLOCKS_SLOT: U256 = U256::from_limbs([1, 0, 0, 0]);
pub const NEXT_EPOCH_BLOCK_SLOT: U256 = U256::from_limbs([2, 0, 0, 0]);
pub const EPOCH_START_BLOCK_MAPPING_SLOT: U256 = U256::from_limbs([3, 0, 0, 0]);

#[cfg(test)]
mod tests;
