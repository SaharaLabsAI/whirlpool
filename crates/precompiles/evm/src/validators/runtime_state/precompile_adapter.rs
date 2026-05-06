use reth_evm::precompiles::PrecompileInput;
use validators_reader::{ValidatorEntry, SIMPLEX_VALIDATORS_REGISTRY};

use crate::validators::{runtime_state::registry_loader, ValidatorsRuntimeError};

pub fn load_active_validator_registry_from_precompile(
    input: &mut PrecompileInput<'_>,
) -> Result<Vec<ValidatorEntry>, ValidatorsRuntimeError> {
    registry_loader::load_active_validator_registry_from_slots(|slot| {
        input
            .internals_mut()
            .sload(SIMPLEX_VALIDATORS_REGISTRY, slot)
            .map(|value| value.data)
            .map_err(|err| err.to_string())
    })
}
