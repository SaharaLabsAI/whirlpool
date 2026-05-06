use std::fmt::Display;

use alloy_primitives::Address;
use state::StateDb;
use validators_reader::{ValidatorEntry, SIMPLEX_VALIDATORS_REGISTRY};

mod precompile_adapter;
mod registry_loader;
pub use precompile_adapter::load_active_validator_registry_from_precompile;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ValidatorsRuntimeError {
    #[error("state access error: {0}")]
    StateAccess(String),
    #[error("malformed validator registry: {0}")]
    MalformedRegistry(String),
    #[error("missing active validator for proposer {proposer_public_key:?}")]
    MissingProposer { proposer_public_key: [u8; 32] },
    #[error("proposer fee recipient mismatch for proposer {proposer_public_key:?}: expected {expected}, got {carried}")]
    FeeRecipientMismatch {
        proposer_public_key: [u8; 32],
        expected: Address,
        carried: Address,
    },
}

pub fn load_active_validator_registry<DB>(
    db: &DB,
) -> Result<Vec<ValidatorEntry>, ValidatorsRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    self::registry_loader::load_active_validator_registry_from_slots(|slot| {
        db.get_storage(SIMPLEX_VALIDATORS_REGISTRY, slot)
            .map_err(|err| err.to_string())
    })
}

pub fn resolve_active_validator_fee_recipient<DB>(
    db: &DB,
    proposer_public_key: [u8; 32],
) -> Result<Address, ValidatorsRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    resolve_active_validator_fee_recipient_from_registry(
        &load_active_validator_registry(db)?,
        proposer_public_key,
    )
}

pub fn validate_active_validator_fee_recipient<DB>(
    db: &DB,
    proposer_public_key: [u8; 32],
    carried_fee_recipient: [u8; 20],
) -> Result<Address, ValidatorsRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    let expected = resolve_active_validator_fee_recipient(db, proposer_public_key)?;
    let carried = Address::from(carried_fee_recipient);
    if expected != carried {
        return Err(ValidatorsRuntimeError::FeeRecipientMismatch {
            proposer_public_key,
            expected,
            carried,
        });
    }
    Ok(expected)
}

fn resolve_active_validator_fee_recipient_from_registry(
    validators: &[ValidatorEntry],
    proposer_public_key: [u8; 32],
) -> Result<Address, ValidatorsRuntimeError> {
    validators
        .iter()
        .find(|entry| entry.consensus_pubkey == proposer_public_key)
        .map(|entry| entry.ethereum_address)
        .ok_or(ValidatorsRuntimeError::MissingProposer {
            proposer_public_key,
        })
}

#[cfg(test)]
mod tests;
