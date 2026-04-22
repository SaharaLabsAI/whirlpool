mod address_storage;
mod registry_codec;

use alloy_primitives::{Address, B256, U256};
use std::collections::BTreeMap;

/// Dedicated genesis account storing ordered simplex validator entries.
pub const SIMPLEX_VALIDATORS_REGISTRY: Address = Address::new([
    0x76, 0x61, 0x6c, 0x69, 0x64, 0x61, 0x74, 0x6f, 0x72, 0x2d, 0x73, 0x65, 0x74, 0x2d, 0x30, 0x31,
    0x2d, 0x6f, 0x72, 0x64,
]);

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

pub use address_storage::encode_ethereum_address_storage_value;
pub use registry_codec::{
    decode_validator_registry_storage, encode_validator_registry_storage, ordered_consensus_pubkeys,
};

pub fn decode_validator_registry_storage_opt(
    storage: Option<&BTreeMap<B256, B256>>,
) -> Result<Vec<ValidatorEntry>, ValidatorRegistryError> {
    match storage {
        Some(storage) => decode_validator_registry_storage(storage),
        None => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    fn b256_from_u64(value: u64) -> B256 {
        B256::from(U256::from(value).to_be_bytes::<32>())
    }

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
    }

    #[test]
    fn registry_reader_returns_full_entries() {
        let entries = vec![ValidatorEntry {
            consensus_pubkey: [0x44; 32],
            ethereum_address: address!("0x0000000000000000000000000000000000000044"),
        }];

        let decoded =
            decode_validator_registry_storage(&encode_validator_registry_storage(&entries))
                .expect("decode registry");

        assert_eq!(decoded[0].consensus_pubkey, [0x44; 32]);
        assert_eq!(
            decoded[0].ethereum_address,
            address!("0x0000000000000000000000000000000000000044")
        );
    }

    #[test]
    fn registry_empty_list_behavior() {
        let decoded = decode_validator_registry_storage_opt(None).expect("decode empty registry");

        assert!(decoded.is_empty());
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
