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
        if !crate::invariants::registry::address_not_already_registered(seen.contains(&address)) {
            return Err(RegistryError::DuplicateCustomAddress(address));
        }
        seen.insert(address);
        if !crate::invariants::registry::address_does_not_collide_with_builtin(
            precompiles.get(&address).is_some(),
        ) {
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
    let installed = installed_precompiles();
    let installed_addresses: Vec<_> = installed
        .iter()
        .map(RegisteredPrecompile::address)
        .collect();
    debug_assert!(crate::invariants::registry::addresses_are_unique(
        &installed_addresses
    ));
    build_precompiles(spec, installed)
}
