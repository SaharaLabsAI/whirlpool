use reth_evm::precompiles::PrecompilesMap;
use reth_evm::revm::primitives::hardfork::SpecId;
use validators_reader::ValidatorEntry as RegistryValidatorEntry;

use crate::build_whirlpool_precompiles_with_validators;

/// Builds a Whirlpool registry.
///
/// This helper is kept for compatibility and minimal bootstrap/test scenarios.
/// Validators runtime reads come from EVM state.
pub fn whirlpool_precompiles(spec: SpecId) -> PrecompilesMap {
    whirlpool_precompiles_with_validators(spec, Vec::new())
}

/// Compatibility constructor for validator-aware callers.
pub fn whirlpool_precompiles_with_validators(
    spec: SpecId,
    _simplex_validators: Vec<RegistryValidatorEntry>,
) -> PrecompilesMap {
    build_whirlpool_precompiles_with_validators(spec, Vec::new())
        .expect("Whirlpool custom precompile registry must be valid")
}
