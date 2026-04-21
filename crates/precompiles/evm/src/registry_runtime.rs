use ::validators::ValidatorEntry as RegistryValidatorEntry;
use reth_evm::precompiles::PrecompilesMap;
use reth_evm::revm::primitives::hardfork::SpecId;

use super::build_whirlpool_precompiles_with_validators;

pub fn whirlpool_precompiles(spec: SpecId) -> PrecompilesMap {
    whirlpool_precompiles_with_validators(spec, Vec::new())
}

pub fn whirlpool_precompiles_with_validators(
    spec: SpecId,
    simplex_validators: Vec<RegistryValidatorEntry>,
) -> PrecompilesMap {
    build_whirlpool_precompiles_with_validators(spec, simplex_validators)
        .expect("Whirlpool custom precompile registry must be valid")
}
