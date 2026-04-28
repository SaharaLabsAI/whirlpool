use alloy_primitives::Address;

use crate::{config::WhirlpoolEvmConfig, error::EvmAppError};

pub fn validate_or_recover_fee_recipient(
    evm_config: &WhirlpoolEvmConfig,
    proposer_public_key: [u8; 32],
    carried_fee_recipient: [u8; 20],
) -> Result<Address, EvmAppError> {
    let carried_fee_recipient = Address::from(carried_fee_recipient);
    match evm_config.fee_recipient_for_proposer(proposer_public_key) {
        Some(expected) if expected != carried_fee_recipient => Err(EvmAppError::InvalidBlock(
            format!(
                "proposer fee recipient mismatch for proposer {:?}: expected {expected}, got {carried_fee_recipient}",
                proposer_public_key
            ),
        )),
        Some(expected) => Ok(expected),
        None => Ok(carried_fee_recipient),
    }
}
