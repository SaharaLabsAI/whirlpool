use std::collections::HashSet;
use std::fmt::Display;

use alloy_primitives::{Address, B256, U256};
use reth_evm::precompiles::PrecompileInput;
use state::StateDb;
use validators_reader::{
    consensus_pubkey_slot, decode_ethereum_address_storage_value, ethereum_address_slot,
    registry_len_slot, ValidatorEntry, ValidatorRegistryError, SIMPLEX_VALIDATORS_REGISTRY,
};

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
    load_active_validator_registry_from_slots(|slot| {
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

pub(crate) fn load_active_validator_registry_from_precompile(
    input: &mut PrecompileInput<'_>,
) -> Result<Vec<ValidatorEntry>, ValidatorsRuntimeError> {
    load_active_validator_registry_from_slots(|slot| {
        input
            .internals_mut()
            .sload(SIMPLEX_VALIDATORS_REGISTRY, slot)
            .map(|value| value.data)
            .map_err(|err| err.to_string())
    })
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

fn load_active_validator_registry_from_slots<F, E>(
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
        if ethereum_address == Address::ZERO {
            return Err(malformed(format!(
                "zero ethereum address at validator index {index}"
            )));
        }

        entries.push(ValidatorEntry {
            consensus_pubkey,
            ethereum_address,
        });
    }

    Ok(entries)
}

fn map_registry_error(err: ValidatorRegistryError) -> ValidatorsRuntimeError {
    ValidatorsRuntimeError::MalformedRegistry(err.to_string())
}

fn malformed(message: impl Into<String>) -> ValidatorsRuntimeError {
    ValidatorsRuntimeError::MalformedRegistry(message.into())
}

fn slot_to_u256(slot: B256) -> U256 {
    U256::from_be_bytes(slot.0)
}

fn storage_word_to_b256(value: U256) -> B256 {
    B256::from(value.to_be_bytes::<32>())
}

fn storage_word_to_bytes(value: U256) -> [u8; 32] {
    value.to_be_bytes::<32>()
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{address, U256};
    use app_evm_state::InMemoryStateDb;
    use validators_reader::{
        encode_validator_registry_storage, ethereum_address_slot, registry_len_slot,
    };

    use super::*;

    fn seed_registry(db: &mut InMemoryStateDb, entries: &[ValidatorEntry]) {
        for (slot, value) in encode_validator_registry_storage(entries) {
            db.insert_storage(
                SIMPLEX_VALIDATORS_REGISTRY,
                slot_to_u256(slot),
                U256::from_be_bytes(value.0),
            );
        }
    }

    #[test]
    fn loads_active_validator_registry_from_statedb() {
        let entries = vec![ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: address!("0x0000000000000000000000000000000000000011"),
        }];
        let mut db = InMemoryStateDb::new();
        seed_registry(&mut db, &entries);

        let decoded = load_active_validator_registry(&db).expect("load registry");

        assert_eq!(decoded, entries);
    }

    #[test]
    fn count_zero_is_empty_active_registry() {
        let db = InMemoryStateDb::new();

        assert_eq!(load_active_validator_registry(&db), Ok(Vec::new()));
        assert_eq!(
            resolve_active_validator_fee_recipient(&db, [0x11; 32]),
            Err(ValidatorsRuntimeError::MissingProposer {
                proposer_public_key: [0x11; 32]
            })
        );
    }

    #[test]
    fn missing_proposer_rejects_lookup() {
        let mut db = InMemoryStateDb::new();
        seed_registry(
            &mut db,
            &[ValidatorEntry {
                consensus_pubkey: [0x11; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000011"),
            }],
        );

        assert_eq!(
            resolve_active_validator_fee_recipient(&db, [0x22; 32]),
            Err(ValidatorsRuntimeError::MissingProposer {
                proposer_public_key: [0x22; 32]
            })
        );
    }

    #[test]
    fn duplicate_pubkey_is_malformed() {
        let mut db = InMemoryStateDb::new();
        seed_registry(
            &mut db,
            &[
                ValidatorEntry {
                    consensus_pubkey: [0x11; 32],
                    ethereum_address: address!("0x0000000000000000000000000000000000000011"),
                },
                ValidatorEntry {
                    consensus_pubkey: [0x11; 32],
                    ethereum_address: address!("0x0000000000000000000000000000000000000022"),
                },
            ],
        );

        let err = load_active_validator_registry(&db).expect_err("duplicate should fail");

        assert!(
            matches!(err, ValidatorsRuntimeError::MalformedRegistry(ref msg) if msg.contains("duplicate consensus pubkey"))
        );
    }

    #[test]
    fn zero_pubkey_is_malformed_when_count_is_nonzero() {
        let mut db = InMemoryStateDb::new();
        db.insert_storage(
            SIMPLEX_VALIDATORS_REGISTRY,
            slot_to_u256(registry_len_slot()),
            U256::from(1_u64),
        );
        db.insert_storage(
            SIMPLEX_VALIDATORS_REGISTRY,
            slot_to_u256(ethereum_address_slot(0)),
            U256::from(1_u64),
        );

        let err = load_active_validator_registry(&db).expect_err("zero pubkey should fail");

        assert!(
            matches!(err, ValidatorsRuntimeError::MalformedRegistry(ref msg) if msg.contains("zero consensus pubkey"))
        );
    }

    #[test]
    fn zero_address_is_malformed_when_count_is_nonzero() {
        let mut db = InMemoryStateDb::new();
        db.insert_storage(
            SIMPLEX_VALIDATORS_REGISTRY,
            slot_to_u256(registry_len_slot()),
            U256::from(1_u64),
        );
        db.insert_storage(
            SIMPLEX_VALIDATORS_REGISTRY,
            slot_to_u256(consensus_pubkey_slot(0)),
            U256::from_be_bytes([0x11; 32]),
        );

        let err = load_active_validator_registry(&db).expect_err("zero address should fail");

        assert!(
            matches!(err, ValidatorsRuntimeError::MalformedRegistry(ref msg) if msg.contains("zero ethereum address"))
        );
    }

    #[test]
    fn invalid_address_padding_is_malformed() {
        let mut db = InMemoryStateDb::new();
        seed_registry(
            &mut db,
            &[ValidatorEntry {
                consensus_pubkey: [0x11; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000011"),
            }],
        );
        let mut padded = [0u8; 32];
        padded[0] = 1;
        padded[31] = 1;
        db.insert_storage(
            SIMPLEX_VALIDATORS_REGISTRY,
            slot_to_u256(ethereum_address_slot(0)),
            U256::from_be_bytes(padded),
        );

        let err = load_active_validator_registry(&db).expect_err("padding should fail");

        assert!(
            matches!(err, ValidatorsRuntimeError::MalformedRegistry(ref msg) if msg.contains("invalid ethereum address storage value"))
        );
    }

    #[test]
    fn registry_length_overflow_is_malformed() {
        let mut db = InMemoryStateDb::new();
        db.insert_storage(
            SIMPLEX_VALIDATORS_REGISTRY,
            slot_to_u256(registry_len_slot()),
            U256::MAX,
        );

        let err = load_active_validator_registry(&db).expect_err("overflow should fail");

        assert!(
            matches!(err, ValidatorsRuntimeError::MalformedRegistry(ref msg) if msg.contains("registry length does not fit"))
        );
    }
}
