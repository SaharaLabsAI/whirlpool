use alloy_primitives::{Address, Bytes};
use alloy_sol_types::{sol, SolCall};
use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};
use validators_reader::ValidatorEntry;

use crate::RegisteredPrecompile;

pub mod gas;
mod precompile_reader;
mod registry_loader;
mod runtime_reader;

pub use runtime_reader::{
    load_active_validator_registry, resolve_active_validator_fee_recipient,
    validate_active_validator_fee_recipient, ValidatorsRuntimeError,
};

pub const VALIDATORS_PRECOMPILE_ADDRESS: Address =
    alloy_primitives::address!("0x0000000000000000000000000000000000000101");

sol! {
    struct ValidatorRecord {
        bytes32 consensusPubkey;
        address ethereumAddress;
    }

    function validators() external view returns (ValidatorRecord[] memory);
}

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

pub fn validators_calldata() -> Bytes {
    Bytes::from(validatorsCall {}.abi_encode())
}

pub fn decode_validators_output(
    payload: &Bytes,
) -> Result<Vec<ValidatorEntry>, ValidatorsPrecompileError> {
    let decoded = validatorsCall::abi_decode_returns(payload.as_ref())
        .map_err(|_| ValidatorsPrecompileError::InvalidReturnPayload)?;

    Ok(decoded
        .into_iter()
        .map(|entry| ValidatorEntry {
            consensus_pubkey: entry.consensusPubkey.0,
            ethereum_address: entry.ethereumAddress,
        })
        .collect())
}

pub fn register(_simplex_validators: Vec<ValidatorEntry>) -> RegisteredPrecompile {
    RegisteredPrecompile::new_stateful(
        "whirlpool_simplex_validators",
        VALIDATORS_PRECOMPILE_ADDRESS,
        execute,
    )
}

fn execute(mut input: PrecompileInput<'_>) -> PrecompileResult {
    decode_call(input.data())?;
    let validators = precompile_reader::load_active_validator_registry_from_precompile(&mut input)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    let gas_cost = gas::validators_gas(validators.len());
    if input.gas() < gas_cost {
        return Err(PrecompileError::OutOfGas);
    }

    Ok(PrecompileOutput::new(
        gas_cost,
        encode_validators_output(&validators),
    ))
}

fn decode_call(data: &[u8]) -> Result<(), PrecompileError> {
    if data.len() < 4 {
        return Err(PrecompileError::other(
            ValidatorsPrecompileError::CalldataTooShort.to_string(),
        ));
    }

    if !data.starts_with(&validatorsCall::SELECTOR) {
        return Err(PrecompileError::other(
            ValidatorsPrecompileError::UnsupportedSelector.to_string(),
        ));
    }

    validatorsCall::abi_decode_validate(data)
        .map(|_| ())
        .map_err(|_| PrecompileError::other(ValidatorsPrecompileError::InvalidCalldata.to_string()))
}

fn encode_validators_output(simplex_validators: &[ValidatorEntry]) -> Bytes {
    let records = simplex_validators
        .iter()
        .map(|entry| ValidatorRecord {
            consensusPubkey: entry.consensus_pubkey.into(),
            ethereumAddress: entry.ethereum_address,
        })
        .collect::<Vec<_>>();

    Bytes::from(validatorsCall::abi_encode_returns(&records))
}

#[cfg(test)]
mod tests;
