use alloy_primitives::U256;

use crate::epoch::storage::*;

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
