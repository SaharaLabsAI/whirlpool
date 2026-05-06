use alloy_primitives::U256;

pub fn decode_u64_storage_value(value: U256) -> Option<u64> {
    u64::try_from(value).ok()
}

pub fn decode_epoch_start_block_storage_value(value: U256) -> Option<u64> {
    let plus_one = decode_u64_storage_value(value)?;
    plus_one.checked_sub(1)
}
