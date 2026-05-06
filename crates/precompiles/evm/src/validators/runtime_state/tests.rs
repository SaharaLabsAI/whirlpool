use alloy_primitives::{address, B256, U256};
use app_evm_state::InMemoryStateDb;
use validators_reader::{
    consensus_pubkey_slot, encode_validator_registry_storage, ethereum_address_slot,
    registry_len_slot, ValidatorEntry, SIMPLEX_VALIDATORS_REGISTRY,
};

use crate::validators::{
    load_active_validator_registry, resolve_active_validator_fee_recipient, ValidatorsRuntimeError,
};

fn slot_to_u256(slot: B256) -> U256 {
    U256::from_be_bytes(slot.0)
}

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
