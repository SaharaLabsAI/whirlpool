use reth_chainspec::ChainSpec;
use validators::{
    decode_validator_registry_storage_opt, ValidatorEntry, ValidatorRegistryError,
    SIMPLEX_VALIDATORS_REGISTRY,
};

pub fn try_simplex_validators_from_chain_spec(
    chain_spec: &ChainSpec,
) -> Result<Vec<ValidatorEntry>, ValidatorRegistryError> {
    decode_validator_registry_storage_opt(
        chain_spec
            .genesis
            .alloc
            .get(&SIMPLEX_VALIDATORS_REGISTRY)
            .and_then(|account| account.storage.as_ref()),
    )
}
