use alloy_primitives::{address, Address, B256, U256};
use std::collections::BTreeMap;

pub const SIMPLEX_VALIDATORS_REGISTRY: Address =
    address!("0x76616c696461746f722d7365742d30312d6f7264");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorEntry {
    pub consensus_pubkey: [u8; 32],
    pub ethereum_address: Address,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ValidatorRegistryError {
    #[error("validator registry length {length} does not fit in usize")]
    RegistryLengthOverflow { length: U256 },
    #[error("missing consensus pubkey storage slot for validator index {index}")]
    MissingConsensusPubkey { index: usize },
    #[error("missing ethereum address storage slot for validator index {index}")]
    MissingEthereumAddress { index: usize },
    #[error("invalid ethereum address storage value for validator index {index}")]
    InvalidEthereumAddressValue { index: usize },
}

pub fn ordered_consensus_pubkeys(entries: &[ValidatorEntry]) -> Vec<[u8; 32]> {
    entries
        .iter()
        .map(|entry| entry.consensus_pubkey)
        .collect::<Vec<_>>()
}

pub fn encode_ethereum_address_storage_value(address: Address) -> B256 {
    let mut bytes = [0u8; 32];
    bytes[12..].copy_from_slice(address.as_slice());
    B256::from(bytes)
}

pub fn encode_validator_registry_storage(entries: &[ValidatorEntry]) -> BTreeMap<B256, B256> {
    let mut storage = BTreeMap::new();
    storage.insert(registry_len_slot(), b256_from_u64(entries.len() as u64));

    for (index, entry) in entries.iter().enumerate() {
        storage.insert(
            consensus_pubkey_slot(index),
            B256::from(entry.consensus_pubkey),
        );
        storage.insert(
            ethereum_address_slot(index),
            encode_ethereum_address_storage_value(entry.ethereum_address),
        );
    }

    storage
}

pub fn decode_validator_registry_storage_opt(
    storage: Option<&BTreeMap<B256, B256>>,
) -> Result<Vec<ValidatorEntry>, ValidatorRegistryError> {
    match storage {
        Some(storage) => decode_validator_registry_storage(storage),
        None => Ok(Vec::new()),
    }
}

pub fn decode_validator_registry_storage(
    storage: &BTreeMap<B256, B256>,
) -> Result<Vec<ValidatorEntry>, ValidatorRegistryError> {
    let count = decode_registry_len(
        storage
            .get(&registry_len_slot())
            .copied()
            .unwrap_or_default(),
    )?;
    let mut entries = Vec::with_capacity(count);

    for index in 0..count {
        let consensus_pubkey = storage
            .get(&consensus_pubkey_slot(index))
            .copied()
            .ok_or(ValidatorRegistryError::MissingConsensusPubkey { index })?
            .0;
        let ethereum_address = decode_ethereum_address_storage_value(
            storage
                .get(&ethereum_address_slot(index))
                .copied()
                .ok_or(ValidatorRegistryError::MissingEthereumAddress { index })?,
            index,
        )?;

        entries.push(ValidatorEntry {
            consensus_pubkey,
            ethereum_address,
        });
    }

    Ok(entries)
}

pub fn registry_len_slot() -> B256 {
    B256::ZERO
}

pub fn consensus_pubkey_slot(index: usize) -> B256 {
    b256_from_u64((index as u64) * 2 + 1)
}

pub fn ethereum_address_slot(index: usize) -> B256 {
    b256_from_u64((index as u64) * 2 + 2)
}

fn decode_registry_len(value: B256) -> Result<usize, ValidatorRegistryError> {
    let len = U256::from_be_bytes(value.0);
    usize::try_from(len).map_err(|_| ValidatorRegistryError::RegistryLengthOverflow { length: len })
}

pub fn decode_ethereum_address_storage_value(
    value: B256,
    index: usize,
) -> Result<Address, ValidatorRegistryError> {
    let bytes = value.as_slice();
    if bytes[..12].iter().any(|byte| *byte != 0) {
        return Err(ValidatorRegistryError::InvalidEthereumAddressValue { index });
    }
    Ok(Address::from_slice(&bytes[12..]))
}

fn b256_from_u64(value: u64) -> B256 {
    B256::from(U256::from(value).to_be_bytes::<32>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    #[test]
    fn registry_round_trip_preserves_order() {
        let entries = vec![
            ValidatorEntry {
                consensus_pubkey: [0x33; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000011"),
            },
            ValidatorEntry {
                consensus_pubkey: [0x11; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000022"),
            },
            ValidatorEntry {
                consensus_pubkey: [0x22; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000033"),
            },
        ];

        let storage = encode_validator_registry_storage(&entries);
        let decoded = decode_validator_registry_storage(&storage).expect("decode registry");

        assert_eq!(decoded, entries);
        assert_eq!(
            ordered_consensus_pubkeys(&entries),
            vec![[0x33; 32], [0x11; 32], [0x22; 32]]
        );
    }

    #[test]
    fn registry_slot_helpers_match_encoded_layout() {
        let entries = vec![ValidatorEntry {
            consensus_pubkey: [0x44; 32],
            ethereum_address: address!("0x0000000000000000000000000000000000000044"),
        }];
        let storage = encode_validator_registry_storage(&entries);

        assert_eq!(
            storage.get(&registry_len_slot()),
            Some(&b256_from_u64(entries.len() as u64))
        );
        assert_eq!(
            storage.get(&consensus_pubkey_slot(0)),
            Some(&B256::from([0x44; 32]))
        );
        assert_eq!(
            storage.get(&ethereum_address_slot(0)),
            Some(&encode_ethereum_address_storage_value(
                entries[0].ethereum_address
            ))
        );
    }

    #[test]
    fn registry_empty_missing_storage_decodes_empty() {
        assert_eq!(
            decode_validator_registry_storage(&BTreeMap::new()),
            Ok(vec![])
        );
        assert_eq!(decode_validator_registry_storage_opt(None), Ok(vec![]));
    }

    #[test]
    fn invalid_address_value_is_rejected() {
        let mut storage = encode_validator_registry_storage(&[ValidatorEntry {
            consensus_pubkey: [0x55; 32],
            ethereum_address: address!("0x0000000000000000000000000000000000000001"),
        }]);
        storage.insert(
            b256_from_u64(2),
            B256::from([
                0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 1,
            ]),
        );

        let err = decode_validator_registry_storage(&storage)
            .expect_err("non-zero address padding should fail");
        assert_eq!(
            err,
            ValidatorRegistryError::InvalidEthereumAddressValue { index: 0 }
        );
    }
}
