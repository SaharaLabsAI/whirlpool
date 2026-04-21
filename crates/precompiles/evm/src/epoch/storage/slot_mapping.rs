use alloy_primitives::{keccak256, U256};

use super::EPOCH_START_BLOCK_MAPPING_SLOT;

/// Deterministic slot for `mapping(uint64 => uint256) epochStartBlockPlusOne`.
pub fn epoch_start_block_slot(epoch: u64) -> U256 {
    let mut encoded = [0u8; 64];
    encoded[24..32].copy_from_slice(&epoch.to_be_bytes());
    encoded[32..64].copy_from_slice(&EPOCH_START_BLOCK_MAPPING_SLOT.to_be_bytes::<32>());
    U256::from_be_bytes(keccak256(encoded).0)
}
