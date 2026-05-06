use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::PrecompileError;

use crate::epoch::{transition::advance::AdvanceEpochEffect, EPOCH_PRECOMPILE_ADDRESS};

pub fn apply_advance_epoch_effect(
    input: &mut PrecompileInput<'_>,
    effect: AdvanceEpochEffect,
) -> Result<(), PrecompileError> {
    for write in effect.writes {
        input
            .internals_mut()
            .sstore(EPOCH_PRECOMPILE_ADDRESS, write.slot, write.value)
            .map_err(|err| PrecompileError::other(err.to_string()))?;
    }

    Ok(())
}
