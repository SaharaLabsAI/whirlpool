use alloy_primitives::Address;
use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};
use validators_reader::ValidatorEntry;

use crate::RegisteredPrecompile;

mod codec;
pub mod gas;
mod runtime_state;

pub use codec::{decode_validators_output, validators_calldata};
pub use runtime_state::{
    load_active_validator_registry, resolve_active_validator_fee_recipient,
    validate_active_validator_fee_recipient, ValidatorsRuntimeError,
};

pub const VALIDATORS_PRECOMPILE_ADDRESS: Address =
    alloy_primitives::address!("0x0000000000000000000000000000000000000101");

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ValidatorsPrecompileError {
    #[error("calldata is too short")]
    CalldataTooShort,
    #[error("unsupported validators selector")]
    UnsupportedSelector,
    #[error("invalid validators calldata")]
    InvalidCalldata,
    #[error("invalid validators return payload")]
    InvalidReturnPayload,
}

pub fn register(_simplex_validators: Vec<ValidatorEntry>) -> RegisteredPrecompile {
    RegisteredPrecompile::new_stateful(
        "whirlpool_simplex_validators",
        VALIDATORS_PRECOMPILE_ADDRESS,
        execute,
    )
}

fn execute(mut input: PrecompileInput<'_>) -> PrecompileResult {
    codec::decode_call(input.data())?;
    let validators = runtime_state::load_active_validator_registry_from_precompile(&mut input)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    let gas_cost = gas::validators_gas(validators.len());
    if input.gas() < gas_cost {
        return Err(PrecompileError::OutOfGas);
    }

    Ok(PrecompileOutput::new(
        gas_cost,
        codec::encode_validators_output(&validators),
    ))
}

#[cfg(test)]
mod tests;
