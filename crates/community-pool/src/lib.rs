use alloy_primitives::Address;

pub const COMMUNITY_POOL_ADDRESS: Address = Address::new([
    0x63, 0x6f, 0x6d, 0x6d, 0x75, 0x6e, 0x69, 0x74, 0x79, 0x2d, 0x70, 0x6f, 0x6f, 0x6c, 0x2d, 0x61,
    0x63, 0x63, 0x6f, 0x75,
]);

#[cfg(test)]
mod tests {
    use super::COMMUNITY_POOL_ADDRESS;

    #[test]
    fn community_pool_address_is_non_zero() {
        assert_ne!(COMMUNITY_POOL_ADDRESS, alloy_primitives::Address::ZERO);
    }
}
