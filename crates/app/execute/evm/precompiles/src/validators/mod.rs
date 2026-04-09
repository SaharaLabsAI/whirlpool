use alloy_primitives::{address, Address, Bytes};
use alloy_sol_types::{sol, SolCall};
use reth_evm::precompiles::PrecompileInput;
use revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};

use crate::RegisteredPrecompile;
use ::validators::ValidatorEntry;

pub const VALIDATORS_PRECOMPILE_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000101");

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

pub fn register(simplex_validators: Vec<ValidatorEntry>) -> RegisteredPrecompile {
    RegisteredPrecompile::new_stateful(
        "whirlpool_simplex_validators",
        VALIDATORS_PRECOMPILE_ADDRESS,
        move |input| execute(input, &simplex_validators),
    )
}

fn execute(input: PrecompileInput<'_>, simplex_validators: &[ValidatorEntry]) -> PrecompileResult {
    let gas_limit = input.gas();
    let gas_cost = gas::validators_gas(simplex_validators.len());
    if gas_limit < gas_cost {
        return Err(PrecompileError::OutOfGas);
    }

    decode_call(input.data())?;

    Ok(PrecompileOutput::new(
        gas_cost,
        encode_validators_output(simplex_validators),
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

pub mod gas {
    pub const BASE_VALIDATORS_GAS: u64 = 3_000;
    pub const PER_VALIDATOR_GAS: u64 = 350;

    pub fn validators_gas(entries: usize) -> u64 {
        BASE_VALIDATORS_GAS.saturating_add(PER_VALIDATOR_GAS.saturating_mul(entries as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, U256};
    use reth_evm::{eth::EthEvmContext, precompiles::Precompile, traits::EvmInternals};
    use revm::{
        context::{BlockEnv, TxEnv},
        database::EmptyDB,
        Context,
    };

    fn call_validators_precompile(
        simplex_validators: Vec<ValidatorEntry>,
        gas: u64,
    ) -> revm::precompile::PrecompileOutput {
        let precompile = register(simplex_validators);
        let mut context: Context<BlockEnv, TxEnv, revm::context::CfgEnv, EmptyDB> =
            EthEvmContext::new(EmptyDB::default(), Default::default());

        precompile
            .precompile()
            .call(PrecompileInput {
                data: validators_calldata().as_ref(),
                gas,
                caller: Address::ZERO,
                value: U256::ZERO,
                target_address: VALIDATORS_PRECOMPILE_ADDRESS,
                bytecode_address: VALIDATORS_PRECOMPILE_ADDRESS,
                is_static: true,
                internals: EvmInternals::from_context(&mut context),
            })
            .expect("validators precompile call should succeed")
    }

    #[test]
    fn validators_precompile_eth_call_returns_full_ordered_registry() {
        let validators = vec![
            ValidatorEntry {
                consensus_pubkey: [0x33; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000033"),
            },
            ValidatorEntry {
                consensus_pubkey: [0x11; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000011"),
            },
        ];

        let result = call_validators_precompile(validators.clone(), gas::validators_gas(2));
        let decoded = decode_validators_output(&result.bytes).expect("decode precompile output");

        assert!(!result.reverted);
        assert_eq!(decoded, validators);
    }

    #[test]
    fn validators_precompile_matches_rust_reader() {
        let source_entries = vec![
            ValidatorEntry {
                consensus_pubkey: [0x55; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000055"),
            },
            ValidatorEntry {
                consensus_pubkey: [0x22; 32],
                ethereum_address: address!("0x0000000000000000000000000000000000000022"),
            },
        ];
        let rust_reader_output = ::validators::decode_validator_registry_storage(
            &::validators::encode_validator_registry_storage(&source_entries),
        )
        .expect("decode rust registry representation");

        let result = call_validators_precompile(rust_reader_output.clone(), gas::validators_gas(2));
        let abi_output = decode_validators_output(&result.bytes).expect("decode precompile output");

        assert_eq!(abi_output, rust_reader_output);
    }

    #[test]
    fn validators_precompile_rejects_bad_selector() {
        let precompile = register(Vec::new()).precompile();
        let mut context: Context<BlockEnv, TxEnv, revm::context::CfgEnv, EmptyDB> =
            EthEvmContext::new(EmptyDB::default(), Default::default());

        let result = precompile
            .call(PrecompileInput {
                data: &[0xde, 0xad, 0xbe, 0xef],
                gas: gas::validators_gas(0),
                caller: Address::ZERO,
                value: U256::ZERO,
                target_address: VALIDATORS_PRECOMPILE_ADDRESS,
                bytecode_address: VALIDATORS_PRECOMPILE_ADDRESS,
                is_static: true,
                internals: EvmInternals::from_context(&mut context),
            })
            .expect_err("unsupported selector should error");

        assert!(result
            .to_string()
            .contains("unsupported validators selector"));
    }
}
