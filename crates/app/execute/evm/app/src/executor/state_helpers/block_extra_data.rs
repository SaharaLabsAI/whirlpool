use alloy_primitives::Address;
use app::{CanonicalExtraDataV1, ExtraDataDecodeMode};

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

pub fn extra_data_decode_mode_for_height(
    evm_config: &WhirlpoolEvmConfig,
    block_height: u64,
) -> ExtraDataDecodeMode {
    if block_height >= evm_config.full_dkg_strict_height() {
        ExtraDataDecodeMode::Strict
    } else {
        ExtraDataDecodeMode::Legacy
    }
}

pub fn proposer_public_key_from_raw_eth_section(
    decoded: &CanonicalExtraDataV1,
) -> Result<[u8; 32], EvmAppError> {
    let Some(raw_eth) = decoded.raw_eth.as_ref() else {
        return Err(EvmAppError::InvalidBlock(
            "missing raw_eth section in block extra_data".into(),
        ));
    };
    if raw_eth.len() != 32 {
        return Err(EvmAppError::InvalidBlock(format!(
            "raw_eth proposer key must be 32 bytes, found {}",
            raw_eth.len()
        )));
    }

    let mut proposer_public_key = [0u8; 32];
    proposer_public_key.copy_from_slice(raw_eth);
    Ok(proposer_public_key)
}
