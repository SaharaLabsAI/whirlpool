use reth_evm::precompiles::PrecompilesMap;
use reth_evm::revm::primitives::hardfork::SpecId;
use validators_reader::ValidatorEntry as RegistryValidatorEntry;

use crate::{build_precompiles, community_pool, epoch, fee_pool, validators, RegistryError};

/// Builds a Whirlpool registry.
///
/// This helper is kept for compatibility and minimal bootstrap/test scenarios.
/// Validator reads come from runtime EVM state.
pub fn build_whirlpool_precompiles(spec: SpecId) -> Result<PrecompilesMap, RegistryError> {
    build_whirlpool_precompiles_with_validators(spec, Vec::new())
}

/// Compatibility constructor for callers that still pass validator entries.
/// Validators runtime reads come from EVM state, not this argument.
pub fn build_whirlpool_precompiles_with_validators(
    spec: SpecId,
    _simplex_validators: Vec<RegistryValidatorEntry>,
) -> Result<PrecompilesMap, RegistryError> {
    build_precompiles(
        spec,
        [
            community_pool::register(),
            epoch::register(),
            fee_pool::register(),
            validators::register(Vec::new()),
        ],
    )
}
