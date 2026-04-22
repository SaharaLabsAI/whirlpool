use alloy_primitives::{Address, B256};

pub fn encode_ethereum_address_storage_value(address: Address) -> B256 {
    let mut bytes = [0u8; 32];
    bytes[12..].copy_from_slice(address.as_slice());
    B256::from(bytes)
}
