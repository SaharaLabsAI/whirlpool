use alloy_primitives::{keccak256, Address, U256};

/// Storage slot discriminator for `mapping(address => uint256) claimable`.
pub const CLAIMABLE_BALANCE_MAPPING_SLOT: U256 = U256::ZERO;

/// Derives the canonical claimable-balance storage slot for a recipient.
///
/// Uses Solidity's mapping addressing rule:
/// `keccak256(pad32(recipient) ++ pad32(mapping_slot))`.
pub fn claimable_balance_slot(recipient: Address) -> U256 {
    let mut encoded = [0u8; 64];
    encoded[12..32].copy_from_slice(recipient.as_slice());
    encoded[32..64].copy_from_slice(&CLAIMABLE_BALANCE_MAPPING_SLOT.to_be_bytes::<32>());
    U256::from_be_bytes(keccak256(encoded).0)
}
