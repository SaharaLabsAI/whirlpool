use alloy_primitives::{Address, Bytes};
use alloy_sol_types::{sol, SolCall};
use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};
use validators_reader::ValidatorEntry;

use crate::RegisteredPrecompile;

pub mod gas;
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
    let validators = runtime_reader::load_active_validator_registry_from_precompile(&mut input)
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
mod tests {
    use super::*;
    use alloy_primitives::{address, U256};
    use reth_evm::revm::{
        context::{BlockEnv, TxEnv},
        database::{CacheDB, EmptyDB},
        Context,
    };
    use reth_evm::{eth::EthEvmContext, precompiles::Precompile, traits::EvmInternals};
    use validators_reader::{
        decode_validator_registry_storage, encode_validator_registry_storage,
        SIMPLEX_VALIDATORS_REGISTRY,
    };

    fn slot_to_u256(slot: alloy_primitives::B256) -> U256 {
        U256::from_be_bytes(slot.0)
    }

    fn seed_registry(db: &mut CacheDB<EmptyDB>, validators: &[ValidatorEntry]) {
        db.insert_account_info(SIMPLEX_VALIDATORS_REGISTRY, Default::default());
        for (slot, value) in encode_validator_registry_storage(validators) {
            db.insert_account_storage(
                SIMPLEX_VALIDATORS_REGISTRY,
                slot_to_u256(slot),
                U256::from_be_bytes(value.0),
            )
            .expect("seed validator registry storage");
        }
    }

    fn call_validators_precompile(
        simplex_validators: Vec<ValidatorEntry>,
        gas: u64,
    ) -> reth_evm::revm::precompile::PrecompileOutput {
        let precompile = register(Vec::new());
        let mut db = CacheDB::<EmptyDB>::default();
        seed_registry(&mut db, &simplex_validators);
        let mut context: Context<
            BlockEnv,
            TxEnv,
            reth_evm::revm::context::CfgEnv,
            CacheDB<EmptyDB>,
        > = EthEvmContext::new(db, Default::default());

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

    fn call_registered_validators_precompile(
        precompile: &RegisteredPrecompile,
        runtime_validators: &[ValidatorEntry],
        gas: u64,
    ) -> reth_evm::revm::precompile::PrecompileOutput {
        let mut db = CacheDB::<EmptyDB>::default();
        seed_registry(&mut db, runtime_validators);
        let mut context: Context<
            BlockEnv,
            TxEnv,
            reth_evm::revm::context::CfgEnv,
            CacheDB<EmptyDB>,
        > = EthEvmContext::new(db, Default::default());

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
        let rust_reader_output =
            decode_validator_registry_storage(&encode_validator_registry_storage(&source_entries))
                .expect("decode rust registry representation");

        let precompile_output = call_validators_precompile(source_entries, gas::validators_gas(2));
        let decoded_precompile =
            decode_validators_output(&precompile_output.bytes).expect("decode precompile output");

        assert_eq!(decoded_precompile, rust_reader_output);
    }

    #[test]
    fn validators_precompile_reads_runtime_storage_not_constructor_snapshot() {
        let constructor_snapshot = vec![ValidatorEntry {
            consensus_pubkey: [0xaa; 32],
            ethereum_address: address!("0x00000000000000000000000000000000000000aa"),
        }];
        let first_runtime_entries = vec![ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: address!("0x0000000000000000000000000000000000000011"),
        }];
        let second_runtime_entries = vec![ValidatorEntry {
            consensus_pubkey: [0x22; 32],
            ethereum_address: address!("0x0000000000000000000000000000000000000022"),
        }];
        let precompile = register(constructor_snapshot);

        let first_output = call_registered_validators_precompile(
            &precompile,
            &first_runtime_entries,
            gas::validators_gas(1),
        );
        let second_output = call_registered_validators_precompile(
            &precompile,
            &second_runtime_entries,
            gas::validators_gas(1),
        );

        assert_eq!(
            decode_validators_output(&first_output.bytes).expect("decode first runtime output"),
            first_runtime_entries
        );
        assert_eq!(
            decode_validators_output(&second_output.bytes).expect("decode second runtime output"),
            second_runtime_entries
        );
    }

    #[test]
    fn validators_precompile_rejects_underpriced_calls() {
        let validators = vec![ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: address!("0x0000000000000000000000000000000000000011"),
        }];
        let precompile = register(Vec::new());
        let mut db = CacheDB::<EmptyDB>::default();
        seed_registry(&mut db, &validators);
        let mut context: Context<
            BlockEnv,
            TxEnv,
            reth_evm::revm::context::CfgEnv,
            CacheDB<EmptyDB>,
        > = EthEvmContext::new(db, Default::default());

        let err = precompile
            .precompile()
            .call(PrecompileInput {
                data: validators_calldata().as_ref(),
                gas: gas::validators_gas(1) - 1,
                caller: Address::ZERO,
                value: U256::ZERO,
                target_address: VALIDATORS_PRECOMPILE_ADDRESS,
                bytecode_address: VALIDATORS_PRECOMPILE_ADDRESS,
                is_static: true,
                internals: EvmInternals::from_context(&mut context),
            })
            .expect_err("underpriced call should fail");

        assert!(matches!(err, PrecompileError::OutOfGas));
    }
}
