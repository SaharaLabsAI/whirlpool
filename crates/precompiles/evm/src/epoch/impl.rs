use alloy_primitives::U256;
use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};

use super::{
    dispatch::{decode_call, EpochCall},
    encode_u64_word, epoch_system_tx_sender, gas, revert_result, storage, EpochPrecompileError,
    EPOCH_PRECOMPILE_ADDRESS,
};

pub(crate) fn execute(mut input: PrecompileInput<'_>) -> PrecompileResult {
    let gas_limit = input.gas();
    let dispatch = match decode_call(input.data()) {
        Ok(call) => call,
        Err(error) => return revert_result(gas::CURRENT_EPOCH_GAS, error),
    };

    match dispatch {
        EpochCall::CurrentEpoch => current_epoch(&mut input, gas_limit),
        EpochCall::NextEpochBlock => next_epoch_block(&mut input, gas_limit),
        EpochCall::EpochBlocks => epoch_blocks(&mut input, gas_limit),
        EpochCall::EpochStartBlock { epoch } => epoch_start_block(&mut input, gas_limit, epoch),
        EpochCall::AdvanceEpoch => advance_epoch(input, gas_limit),
    }
}

fn current_epoch(input: &mut PrecompileInput<'_>, gas_limit: u64) -> PrecompileResult {
    if gas_limit < gas::CURRENT_EPOCH_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    let value = load_u64_slot(input, storage::current_epoch_slot())?;
    Ok(PrecompileOutput::new(
        gas::CURRENT_EPOCH_GAS,
        encode_u64_word(value),
    ))
}

fn next_epoch_block(input: &mut PrecompileInput<'_>, gas_limit: u64) -> PrecompileResult {
    if gas_limit < gas::NEXT_EPOCH_BLOCK_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    let value = load_u64_slot(input, storage::next_epoch_block_slot())?;
    Ok(PrecompileOutput::new(
        gas::NEXT_EPOCH_BLOCK_GAS,
        encode_u64_word(value),
    ))
}

fn epoch_blocks(input: &mut PrecompileInput<'_>, gas_limit: u64) -> PrecompileResult {
    if gas_limit < gas::EPOCH_BLOCKS_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    let value = load_u64_slot(input, storage::epoch_blocks_slot())?;
    Ok(PrecompileOutput::new(
        gas::EPOCH_BLOCKS_GAS,
        encode_u64_word(value),
    ))
}

fn epoch_start_block(
    input: &mut PrecompileInput<'_>,
    gas_limit: u64,
    epoch: u64,
) -> PrecompileResult {
    if gas_limit < gas::EPOCH_START_BLOCK_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    let raw = input
        .internals_mut()
        .sload(
            EPOCH_PRECOMPILE_ADDRESS,
            storage::epoch_start_block_slot(epoch),
        )
        .map(|value| value.data)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    let decoded = storage::decode_epoch_start_block_storage_value(raw).ok_or_else(|| {
        PrecompileError::other(EpochPrecompileError::EpochNotInitialized(epoch).to_string())
    })?;

    Ok(PrecompileOutput::new(
        gas::EPOCH_START_BLOCK_GAS,
        encode_u64_word(decoded),
    ))
}

fn advance_epoch(mut input: PrecompileInput<'_>, gas_limit: u64) -> PrecompileResult {
    if gas_limit < gas::ADVANCE_EPOCH_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    if input.is_static_call() {
        return revert_result(
            gas::ADVANCE_EPOCH_GAS,
            EpochPrecompileError::StaticCallAdvanceEpoch,
        );
    }

    if *input.caller() != epoch_system_tx_sender() {
        return revert_result(
            gas::ADVANCE_EPOCH_GAS,
            EpochPrecompileError::UnauthorizedAdvanceEpochCaller,
        );
    }

    let current_epoch = load_u64_slot(&mut input, storage::current_epoch_slot())?;
    let next_epoch_block = load_u64_slot(&mut input, storage::next_epoch_block_slot())?;
    let epoch_blocks = load_u64_slot(&mut input, storage::epoch_blocks_slot())?;
    let block_number = u64::try_from(input.internals().block_number())
        .map_err(|_| PrecompileError::other(EpochPrecompileError::ValueOutOfRange.to_string()))?;

    if block_number != next_epoch_block {
        return revert_result(
            gas::ADVANCE_EPOCH_GAS,
            EpochPrecompileError::InvalidBoundaryBlock {
                expected: next_epoch_block,
                got: block_number,
            },
        );
    }

    let next_epoch = current_epoch.checked_add(1).ok_or_else(|| {
        PrecompileError::other(EpochPrecompileError::ArithmeticOverflow.to_string())
    })?;

    let start_slot = storage::epoch_start_block_slot(next_epoch);
    let existing = input
        .internals_mut()
        .sload(EPOCH_PRECOMPILE_ADDRESS, start_slot)
        .map(|value| value.data)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    if existing != U256::ZERO {
        return revert_result(
            gas::ADVANCE_EPOCH_GAS,
            EpochPrecompileError::EpochStartAlreadyInitialized(next_epoch),
        );
    }

    let encoded_start = block_number.checked_add(1).ok_or_else(|| {
        PrecompileError::other(EpochPrecompileError::ArithmeticOverflow.to_string())
    })?;
    let next_boundary = next_epoch_block.checked_add(epoch_blocks).ok_or_else(|| {
        PrecompileError::other(EpochPrecompileError::ArithmeticOverflow.to_string())
    })?;

    input
        .internals_mut()
        .sstore(
            EPOCH_PRECOMPILE_ADDRESS,
            storage::current_epoch_slot(),
            U256::from(next_epoch),
        )
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    input
        .internals_mut()
        .sstore(
            EPOCH_PRECOMPILE_ADDRESS,
            storage::next_epoch_block_slot(),
            U256::from(next_boundary),
        )
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    input
        .internals_mut()
        .sstore(
            EPOCH_PRECOMPILE_ADDRESS,
            start_slot,
            U256::from(encoded_start),
        )
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    Ok(PrecompileOutput::new(
        gas::ADVANCE_EPOCH_GAS,
        Default::default(),
    ))
}

fn load_u64_slot(input: &mut PrecompileInput<'_>, slot: U256) -> Result<u64, PrecompileError> {
    let raw = input
        .internals_mut()
        .sload(EPOCH_PRECOMPILE_ADDRESS, slot)
        .map(|value| value.data)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    storage::decode_u64_storage_value(raw)
        .ok_or_else(|| PrecompileError::other(EpochPrecompileError::ValueOutOfRange.to_string()))
}
