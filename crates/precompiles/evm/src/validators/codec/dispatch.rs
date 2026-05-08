use alloy_sol_types::{sol, SolCall};
use reth_evm::revm::precompile::PrecompileError;

use crate::validators::ValidatorsPrecompileError;

sol! {
    struct ValidatorRecord {
        bytes32 consensusPubkey;
        address ethereumAddress;
    }

    function validators() external view returns (ValidatorRecord[] memory);
}

pub fn decode_call(data: &[u8]) -> Result<(), PrecompileError> {
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
