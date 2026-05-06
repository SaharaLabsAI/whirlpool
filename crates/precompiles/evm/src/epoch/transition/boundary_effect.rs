use alloy_primitives::U256;
use reth_evm::revm::state::EvmState;

use crate::epoch::{
    current_epoch_slot, decode_epoch_start_block_storage_value, decode_u64_storage_value,
    epoch_blocks_slot, epoch_start_block_slot, next_epoch_block_slot, EPOCH_PRECOMPILE_ADDRESS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochBoundaryStorageWrite {
    pub slot: U256,
    pub value: U256,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochBoundaryEffect {
    pub writes: [EpochBoundaryStorageWrite; 3],
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EpochBoundaryEffectError {
    #[error("epoch boundary effect must only touch the epoch precompile account")]
    UnexpectedChangedAccount,
    #[error("epoch boundary effect must not require account-info replay")]
    AccountInfoReplayRequired,
    #[error("epoch boundary effect is missing epoch precompile storage changes")]
    MissingEpochStorageChanges,
    #[error("epoch boundary effect contains duplicate slot {0}")]
    DuplicateChangedSlot(U256),
    #[error("epoch boundary effect contains unexpected slot {0}")]
    UnexpectedChangedSlot(U256),
    #[error("epoch boundary effect is missing currentEpoch write")]
    MissingCurrentEpochWrite,
    #[error("epoch boundary effect is missing nextEpochBlock write")]
    MissingNextEpochBlockWrite,
    #[error("epoch boundary effect is missing epochStartBlock(next_epoch) write")]
    MissingEpochStartBlockWrite,
    #[error("epoch boundary currentEpoch value does not fit in u64")]
    InvalidCurrentEpochValue,
    #[error("epoch boundary original currentEpoch value does not fit in u64")]
    InvalidOriginalCurrentEpochValue,
    #[error("epoch boundary nextEpochBlock value does not fit in u64")]
    InvalidNextEpochBlockValue,
    #[error("epoch boundary currentEpoch write did not increment by one")]
    InvalidCurrentEpochTransition,
    #[error("epoch boundary epochStartBlock value is not storage-ready encoded")]
    InvalidEpochStartBlockEncoding,
    #[error("epoch boundary epochStartBlock slot was already initialized")]
    EpochStartBlockAlreadyInitialized,
}

pub fn extract_epoch_boundary_effect(
    outcome_state: &EvmState,
) -> Result<EpochBoundaryEffect, EpochBoundaryEffectError> {
    let mut epoch_writes = None;
    let mut loaded_epoch_blocks = None;
    let mut loaded_next_epoch_block = None;

    for (address, account) in outcome_state {
        let changed_slots: Vec<_> = account
            .changed_storage_slots()
            .map(|(slot, value)| (*slot, value.original_value(), value.present_value()))
            .collect();
        let info_changed = account.info != *account.original_info;

        if changed_slots.is_empty() && !info_changed {
            continue;
        }

        if *address != EPOCH_PRECOMPILE_ADDRESS {
            return Err(EpochBoundaryEffectError::UnexpectedChangedAccount);
        }

        if info_changed {
            return Err(EpochBoundaryEffectError::AccountInfoReplayRequired);
        }

        loaded_epoch_blocks = account
            .storage
            .get(&epoch_blocks_slot())
            .map(|slot| slot.present_value());
        loaded_next_epoch_block = account
            .storage
            .get(&next_epoch_block_slot())
            .map(|slot| (slot.original_value(), slot.present_value()));
        epoch_writes = Some(changed_slots);
    }

    let changed_slots = epoch_writes.ok_or(EpochBoundaryEffectError::MissingEpochStorageChanges)?;

    let mut current_epoch_write = None;
    let mut next_epoch_block_write = None;
    let mut epoch_start_block_write = None;

    for (slot, original_value, present_value) in changed_slots {
        if slot == current_epoch_slot() {
            record_epoch_storage_write(
                &mut current_epoch_write,
                (original_value, present_value),
                slot,
            )?;
        } else if slot == next_epoch_block_slot() {
            record_epoch_storage_write(
                &mut next_epoch_block_write,
                (original_value, present_value),
                slot,
            )?;
        } else {
            record_epoch_storage_write(
                &mut epoch_start_block_write,
                (slot, original_value, present_value),
                slot,
            )?;
        }
    }

    let (current_epoch_original, current_epoch_value) =
        current_epoch_write.ok_or(EpochBoundaryEffectError::MissingCurrentEpochWrite)?;
    let (epoch_start_slot, epoch_start_original, epoch_start_value) =
        epoch_start_block_write.ok_or(EpochBoundaryEffectError::MissingEpochStartBlockWrite)?;

    let current_epoch = decode_u64_storage_value(current_epoch_value)
        .ok_or(EpochBoundaryEffectError::InvalidCurrentEpochValue)?;
    let original_current_epoch = decode_u64_storage_value(current_epoch_original)
        .ok_or(EpochBoundaryEffectError::InvalidOriginalCurrentEpochValue)?;
    if current_epoch != original_current_epoch.saturating_add(1) {
        return Err(EpochBoundaryEffectError::InvalidCurrentEpochTransition);
    }

    let expected_epoch_start_slot = epoch_start_block_slot(current_epoch);
    if epoch_start_slot != expected_epoch_start_slot {
        return Err(EpochBoundaryEffectError::UnexpectedChangedSlot(
            epoch_start_slot,
        ));
    }

    if epoch_start_original != U256::ZERO {
        return Err(EpochBoundaryEffectError::EpochStartBlockAlreadyInitialized);
    }

    decode_epoch_start_block_storage_value(epoch_start_value)
        .ok_or(EpochBoundaryEffectError::InvalidEpochStartBlockEncoding)?;
    let next_epoch_block_value = match next_epoch_block_write {
        Some((_original, present)) => {
            decode_u64_storage_value(present)
                .ok_or(EpochBoundaryEffectError::InvalidNextEpochBlockValue)?;
            present
        }
        None => {
            let (loaded_next_epoch_original, _) = loaded_next_epoch_block
                .ok_or(EpochBoundaryEffectError::MissingNextEpochBlockWrite)?;
            let loaded_epoch_blocks =
                loaded_epoch_blocks.ok_or(EpochBoundaryEffectError::MissingNextEpochBlockWrite)?;
            let next_epoch_block = decode_u64_storage_value(loaded_next_epoch_original)
                .ok_or(EpochBoundaryEffectError::InvalidNextEpochBlockValue)?;
            let epoch_blocks = decode_u64_storage_value(loaded_epoch_blocks)
                .ok_or(EpochBoundaryEffectError::InvalidNextEpochBlockValue)?;
            let synthesized_next_epoch_block = next_epoch_block
                .checked_add(epoch_blocks)
                .ok_or(EpochBoundaryEffectError::InvalidNextEpochBlockValue)?;
            U256::from(synthesized_next_epoch_block)
        }
    };

    Ok(EpochBoundaryEffect {
        writes: [
            EpochBoundaryStorageWrite {
                slot: current_epoch_slot(),
                value: current_epoch_value,
            },
            EpochBoundaryStorageWrite {
                slot: next_epoch_block_slot(),
                value: next_epoch_block_value,
            },
            EpochBoundaryStorageWrite {
                slot: epoch_start_slot,
                value: epoch_start_value,
            },
        ],
    })
}

fn record_epoch_storage_write<T>(
    target: &mut Option<T>,
    value: T,
    slot: U256,
) -> Result<(), EpochBoundaryEffectError> {
    if target.replace(value).is_some() {
        return Err(EpochBoundaryEffectError::DuplicateChangedSlot(slot));
    }

    Ok(())
}

#[cfg(test)]
#[path = "boundary_effect_tests.rs"]
mod tests;
