use crate::validators::ValidatorEntry as RegistryValidatorEntry;
use reth_evm::precompiles::PrecompilesMap;
use reth_evm::revm::primitives::hardfork::SpecId;

use super::build_whirlpool_precompiles_with_validators;

/// Builds a Whirlpool registry with an empty validator snapshot.
///
/// This helper is kept for compatibility and minimal bootstrap/test scenarios.
/// It is **not** the canonical runtime wiring path because the validators
/// precompile will expose an empty registry. Production EVM wiring should use
/// [`whirlpool_precompiles_with_validators`] so the ordered simplex-validator
/// list is captured in the registry.
pub fn whirlpool_precompiles(spec: SpecId) -> PrecompilesMap {
    whirlpool_precompiles_with_validators(spec, Vec::new())
}

/// Builds the canonical validator-aware Whirlpool registry used by runtime EVM wiring.
pub fn whirlpool_precompiles_with_validators(
    spec: SpecId,
    simplex_validators: Vec<RegistryValidatorEntry>,
) -> PrecompilesMap {
    build_whirlpool_precompiles_with_validators(spec, simplex_validators)
        .expect("Whirlpool custom precompile registry must be valid")
}
