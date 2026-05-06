use alloy_primitives::U256;
use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::PrecompileError;

use crate::epoch::{storage, EpochPrecompileError, EPOCH_PRECOMPILE_ADDRESS};

pub fn load_u64_slot(input: &mut PrecompileInput<'_>, slot: U256) -> Result<u64, PrecompileError> {
    let raw = input
        .internals_mut()
        .sload(EPOCH_PRECOMPILE_ADDRESS, slot)
        .map(|value| value.data)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    storage::decode_u64_storage_value(raw)
        .ok_or_else(|| PrecompileError::other(EpochPrecompileError::ValueOutOfRange.to_string()))
}
