use alloy_primitives::Bytes;
use alloy_sol_types::SolCall;
use validators_reader::ValidatorEntry;

use crate::validators::codec::dispatch::{validatorsCall, ValidatorRecord};
use crate::validators::ValidatorsPrecompileError;

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

pub fn encode_validators_output(simplex_validators: &[ValidatorEntry]) -> Bytes {
    let records = simplex_validators
        .iter()
        .map(|entry| ValidatorRecord {
            consensusPubkey: entry.consensus_pubkey.into(),
            ethereumAddress: entry.ethereum_address,
        })
        .collect::<Vec<_>>();

    Bytes::from(validatorsCall::abi_encode_returns(&records))
}
