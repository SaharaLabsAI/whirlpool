use std::collections::HashSet;
use std::fmt::Display;

use alloy_primitives::{B256, U256};
use validators_reader::{
    consensus_pubkey_slot, decode_ethereum_address_storage_value, ethereum_address_slot,
    registry_len_slot, ValidatorEntry, ValidatorRegistryError,
};

use crate::validators::runtime_state::ValidatorsRuntimeError;

pub fn load_active_validator_registry_from_slots<F, E>(
    mut load_slot: F,
) -> Result<Vec<ValidatorEntry>, ValidatorsRuntimeError>
where
    F: FnMut(U256) -> Result<U256, E>,
    E: Display,
{
    let count = usize::try_from(
        load_slot(slot_to_u256(registry_len_slot()))
            .map_err(|err| ValidatorsRuntimeError::StateAccess(err.to_string()))?,
    )
    .map_err(|_| malformed("registry length does not fit in usize"))?;
    let mut entries = Vec::with_capacity(count);
    let mut seen = HashSet::with_capacity(count);

    for index in 0..count {
        let consensus_pubkey = storage_word_to_bytes(
            load_slot(slot_to_u256(consensus_pubkey_slot(index)))
                .map_err(|err| ValidatorsRuntimeError::StateAccess(err.to_string()))?,
        );
        if consensus_pubkey == [0u8; 32] {
            return Err(malformed(format!(
                "zero consensus pubkey at validator index {index}"
            )));
        }
        if !seen.insert(consensus_pubkey) {
            return Err(malformed(format!(
                "duplicate consensus pubkey at validator index {index}"
            )));
        }

        let address_word = storage_word_to_b256(
            load_slot(slot_to_u256(ethereum_address_slot(index)))
                .map_err(|err| ValidatorsRuntimeError::StateAccess(err.to_string()))?,
        );
        let ethereum_address = decode_ethereum_address_storage_value(address_word, index)
            .map_err(map_registry_error)?;
        if ethereum_address == alloy_primitives::Address::ZERO {
            return Err(malformed(format!(
                "zero ethereum address at validator index {index}"
            )));
        }

        entries.push(ValidatorEntry {
            consensus_pubkey,
            ethereum_address,
        });
    }

    if !crate::invariants::validators::active_registry_entries_are_well_formed(&entries) {
        return Err(malformed("active validator registry invariant violation"));
    }

    Ok(entries)
}

fn slot_to_u256(slot: B256) -> U256 {
    U256::from_be_bytes(slot.0)
}

fn map_registry_error(err: ValidatorRegistryError) -> ValidatorsRuntimeError {
    ValidatorsRuntimeError::MalformedRegistry(err.to_string())
}

fn malformed(message: impl Into<String>) -> ValidatorsRuntimeError {
    ValidatorsRuntimeError::MalformedRegistry(message.into())
}

fn storage_word_to_b256(value: U256) -> B256 {
    B256::from(value.to_be_bytes::<32>())
}

fn storage_word_to_bytes(value: U256) -> [u8; 32] {
    value.to_be_bytes::<32>()
}
