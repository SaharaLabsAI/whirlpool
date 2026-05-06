use std::collections::HashSet;

use reth_evm::precompiles::PrecompilesMap;
use reth_evm::revm::{precompile::PrecompileSpecId, primitives::hardfork::SpecId};
use validators_reader::ValidatorEntry as RegistryValidatorEntry;

use crate::registry::installed::installed_precompiles;
use crate::{RegisteredPrecompile, RegistryError};

pub fn build_precompiles<I>(
    spec: SpecId,
    custom_precompiles: I,
) -> Result<PrecompilesMap, RegistryError>
where
    I: IntoIterator<Item = RegisteredPrecompile>,
{
    let mut precompiles = PrecompilesMap::from_static(
        reth_evm::revm::precompile::Precompiles::new(PrecompileSpecId::from_spec_id(spec)),
    );
    let mut seen = HashSet::new();

    for registered in custom_precompiles {
        let address = registered.address();
        if !seen.insert(address) {
            return Err(RegistryError::DuplicateCustomAddress(address));
        }
        if precompiles.get(&address).is_some() {
            return Err(RegistryError::BuiltinAddressCollision(address));
        }
        precompiles.apply_precompile(&address, |_| Some(registered.precompile()));
    }

    Ok(precompiles)
}

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
    build_precompiles(spec, installed_precompiles())
}
