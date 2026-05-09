use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};

use crate::epoch::{
    codec::{decode_call, EpochCall},
    encode_u64_word, epoch_system_tx_sender, gas, revert_result, storage,
    transition::advance::{
        finalize_advance_epoch, plan_advance_epoch, AdvanceEpochInput, AdvanceEpochState,
    },
    EpochPrecompileError, EPOCH_PRECOMPILE_ADDRESS,
};

use crate::epoch::runtime::{effect_writer::apply_advance_epoch_effect, state::load_u64_slot};

pub fn execute(mut input: PrecompileInput<'_>) -> PrecompileResult {
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

    if !crate::invariants::call_boundary::write_call_is_not_static(input.is_static_call()) {
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
    let advance_input = AdvanceEpochInput { block_number };
    let advance_state = AdvanceEpochState {
        current_epoch,
        next_epoch_block,
        epoch_blocks,
    };
    let plan = match plan_advance_epoch(advance_input, advance_state) {
        Ok(plan) => plan,
        Err(error) => return advance_epoch_error(error),
    };

    let existing_epoch_start = input
        .internals_mut()
        .sload(EPOCH_PRECOMPILE_ADDRESS, plan.epoch_start_slot)
        .map(|value| value.data)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    let outcome = match finalize_advance_epoch(plan, existing_epoch_start) {
        Ok(outcome) => outcome,
        Err(error) => return advance_epoch_error(error),
    };
    if !crate::invariants::epoch::advance_effect_is_consistent(
        advance_state.current_epoch,
        advance_state.next_epoch_block,
        advance_state.epoch_blocks,
        advance_input.block_number,
        &outcome.effect.writes,
    ) {
        return Err(PrecompileError::other("epoch advance invariant violation"));
    }

    apply_advance_epoch_effect(&mut input, outcome.effect)?;

    Ok(PrecompileOutput::new(
        gas::ADVANCE_EPOCH_GAS,
        Default::default(),
    ))
}

fn advance_epoch_error(error: EpochPrecompileError) -> PrecompileResult {
    match error {
        EpochPrecompileError::InvalidBoundaryBlock { .. }
        | EpochPrecompileError::EpochStartAlreadyInitialized(_) => {
            revert_result(gas::ADVANCE_EPOCH_GAS, error)
        }
        _ => Err(PrecompileError::other(error.to_string())),
    }
}
