use alloy_primitives::{keccak256, B256, U256};

pub const CURRENT_EPOCH_SLOT: U256 = U256::ZERO;
pub const EPOCH_BLOCKS_SLOT: U256 = U256::from_limbs([1, 0, 0, 0]);
pub const NEXT_EPOCH_BLOCK_SLOT: U256 = U256::from_limbs([2, 0, 0, 0]);
pub const EPOCH_START_BLOCK_MAPPING_SLOT: U256 = U256::from_limbs([3, 0, 0, 0]);

pub fn current_epoch_slot() -> U256 {
    CURRENT_EPOCH_SLOT
}

pub fn epoch_blocks_slot() -> U256 {
    EPOCH_BLOCKS_SLOT
}

pub fn next_epoch_block_slot() -> U256 {
    NEXT_EPOCH_BLOCK_SLOT
}

/// Deterministic slot for `mapping(uint64 => uint256) epochStartBlockPlusOne`.
pub fn epoch_start_block_slot(epoch: u64) -> U256 {
    let mut encoded = [0u8; 64];
    encoded[24..32].copy_from_slice(&epoch.to_be_bytes());
    encoded[32..64].copy_from_slice(&EPOCH_START_BLOCK_MAPPING_SLOT.to_be_bytes::<32>());
    U256::from_be_bytes(keccak256(encoded).0)
}

pub fn current_epoch_storage_slot() -> B256 {
    B256::from(CURRENT_EPOCH_SLOT.to_be_bytes::<32>())
}

pub fn epoch_blocks_storage_slot() -> B256 {
    B256::from(EPOCH_BLOCKS_SLOT.to_be_bytes::<32>())
}

pub fn next_epoch_block_storage_slot() -> B256 {
    B256::from(NEXT_EPOCH_BLOCK_SLOT.to_be_bytes::<32>())
}

pub fn epoch_start_block_storage_slot(epoch: u64) -> B256 {
    B256::from(epoch_start_block_slot(epoch).to_be_bytes::<32>())
}

pub fn encode_u64_storage_value(value: u64) -> B256 {
    B256::from(U256::from(value).to_be_bytes::<32>())
}

pub fn encode_epoch_start_block_storage_value(start_block: u64) -> B256 {
    let plus_one = start_block
        .checked_add(1)
        .expect("epoch start block + 1 must fit into u64");
    encode_u64_storage_value(plus_one)
}

pub fn decode_u64_storage_value(value: U256) -> Option<u64> {
    u64::try_from(value).ok()
}

pub fn decode_epoch_start_block_storage_value(value: U256) -> Option<u64> {
    let plus_one = decode_u64_storage_value(value)?;
    plus_one.checked_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_mapping_slot_is_deterministic() {
        assert_eq!(epoch_start_block_slot(0), epoch_start_block_slot(0));
        assert_ne!(epoch_start_block_slot(0), epoch_start_block_slot(1));
    }

    #[test]
    fn epoch_start_encoding_roundtrip() {
        let encoded = encode_epoch_start_block_storage_value(0);
        let decoded = decode_epoch_start_block_storage_value(U256::from_be_bytes(encoded.0));
        assert_eq!(decoded, Some(0));
    }
}
