use alloy_primitives::U256;

use crate::epoch::{storage, EpochBoundaryStorageWrite, EpochPrecompileError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvanceEpochInput {
    pub block_number: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvanceEpochState {
    pub current_epoch: u64,
    pub next_epoch_block: u64,
    pub epoch_blocks: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceEpochPlan {
    pub next_epoch: u64,
    pub epoch_start_slot: U256,
    writes: [EpochBoundaryStorageWrite; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceEpochEffect {
    pub writes: [EpochBoundaryStorageWrite; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvanceEpochOutcome {
    pub effect: AdvanceEpochEffect,
}

pub fn plan_advance_epoch(
    input: AdvanceEpochInput,
    state: AdvanceEpochState,
) -> Result<AdvanceEpochPlan, EpochPrecompileError> {
    if input.block_number != state.next_epoch_block {
        return Err(EpochPrecompileError::InvalidBoundaryBlock {
            expected: state.next_epoch_block,
            got: input.block_number,
        });
    }

    let next_epoch = state
        .current_epoch
        .checked_add(1)
        .ok_or(EpochPrecompileError::ArithmeticOverflow)?;
    let encoded_start = input
        .block_number
        .checked_add(1)
        .ok_or(EpochPrecompileError::ArithmeticOverflow)?;
    let next_boundary = state
        .next_epoch_block
        .checked_add(state.epoch_blocks)
        .ok_or(EpochPrecompileError::ArithmeticOverflow)?;
    let epoch_start_slot = storage::epoch_start_block_slot(next_epoch);

    Ok(AdvanceEpochPlan {
        next_epoch,
        epoch_start_slot,
        writes: [
            EpochBoundaryStorageWrite {
                slot: storage::current_epoch_slot(),
                value: U256::from(next_epoch),
            },
            EpochBoundaryStorageWrite {
                slot: storage::next_epoch_block_slot(),
                value: U256::from(next_boundary),
            },
            EpochBoundaryStorageWrite {
                slot: epoch_start_slot,
                value: U256::from(encoded_start),
            },
        ],
    })
}

pub fn finalize_advance_epoch(
    plan: AdvanceEpochPlan,
    existing_epoch_start_value: U256,
) -> Result<AdvanceEpochOutcome, EpochPrecompileError> {
    if existing_epoch_start_value != U256::ZERO {
        return Err(EpochPrecompileError::EpochStartAlreadyInitialized(
            plan.next_epoch,
        ));
    }

    Ok(AdvanceEpochOutcome {
        effect: AdvanceEpochEffect {
            writes: plan.writes,
        },
    })
}

#[cfg(test)]
mod tests;
