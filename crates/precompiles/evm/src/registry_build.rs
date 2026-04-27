use crate::validators::ValidatorEntry as RegistryValidatorEntry;
use reth_evm::precompiles::PrecompilesMap;
use reth_evm::revm::primitives::hardfork::SpecId;

use super::{build_precompiles, community_pool, epoch, fee_pool, validators, RegistryError};

/// Builds a Whirlpool registry with an empty validator snapshot.
///
/// This helper is kept for compatibility and minimal bootstrap/test scenarios.
/// It is **not** the canonical runtime path because the validators precompile
/// will expose an empty registry. Runtime callers should prefer
/// [`build_whirlpool_precompiles_with_validators`].
pub fn build_whirlpool_precompiles(spec: SpecId) -> Result<PrecompilesMap, RegistryError> {
    build_whirlpool_precompiles_with_validators(spec, Vec::new())
}

/// Builds the canonical validator-aware Whirlpool registry definition.
pub fn build_whirlpool_precompiles_with_validators(
    spec: SpecId,
    simplex_validators: Vec<RegistryValidatorEntry>,
) -> Result<PrecompilesMap, RegistryError> {
    build_precompiles(
        spec,
        [
            community_pool::register(),
            epoch::register(),
            fee_pool::register(),
            validators::register(simplex_validators),
        ],
    )
}
