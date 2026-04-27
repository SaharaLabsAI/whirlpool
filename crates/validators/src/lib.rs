pub use evm_precompiles::validators::{
    decode_validator_registry_storage, decode_validator_registry_storage_opt,
    encode_ethereum_address_storage_value, encode_validator_registry_storage,
    ordered_consensus_pubkeys, ValidatorEntry, ValidatorRegistryError, SIMPLEX_VALIDATORS_REGISTRY,
};

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, B256, U256};
    use evm_precompiles::validators as canonical;
    use std::collections::BTreeMap;

    fn b256_from_u64(value: u64) -> B256 {
        B256::from(U256::from(value).to_be_bytes::<32>())
    }

    #[test]
    fn wrapper_exports_canonical_registry_address() {
        assert_eq!(
            SIMPLEX_VALIDATORS_REGISTRY,
            canonical::SIMPLEX_VALIDATORS_REGISTRY
        );
    }

    #[test]
    fn wrapper_codec_forwards_to_canonical_implementation() {
        let entries = vec![
            ValidatorEntry {
                consensus_pubkey: [0x33; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000011"),
            },
            ValidatorEntry {
                consensus_pubkey: [0x11; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000022"),
            },
        ];

        let wrapper_storage = encode_validator_registry_storage(&entries);
        let canonical_storage = canonical::encode_validator_registry_storage(&entries);

        assert_eq!(wrapper_storage, canonical_storage);
        assert_eq!(
            decode_validator_registry_storage(&wrapper_storage).expect("wrapper decode"),
            canonical::decode_validator_registry_storage(&canonical_storage)
                .expect("canonical decode")
        );
        assert_eq!(
            ordered_consensus_pubkeys(&entries),
            vec![[0x33; 32], [0x11; 32]]
        );
    }

    #[test]
    fn wrapper_preserves_optional_empty_decode_and_error_shape() {
        assert_eq!(decode_validator_registry_storage_opt(None), Ok(vec![]));

        let storage = BTreeMap::from([
            (b256_from_u64(0), b256_from_u64(1)),
            (b256_from_u64(1), B256::from([0x55; 32])),
            (
                b256_from_u64(2),
                B256::from([
                    0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 1,
                ]),
            ),
        ]);

        assert_eq!(
            decode_validator_registry_storage(&storage),
            Err(ValidatorRegistryError::InvalidEthereumAddressValue { index: 0 })
        );
    }
}
