use alloy_primitives::U256;
use reth_evm::revm::state::EvmState;

use super::{
    current_epoch_slot, decode_epoch_start_block_storage_value, decode_u64_storage_value,
    epoch_start_block_slot, next_epoch_block_slot, EPOCH_PRECOMPILE_ADDRESS,
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

        epoch_writes = Some(changed_slots);
    }

    let changed_slots = epoch_writes.ok_or(EpochBoundaryEffectError::MissingEpochStorageChanges)?;

    let mut current_epoch_write = None;
    let mut next_epoch_block_write = None;
    let mut epoch_start_block_write = None;

    for (slot, original_value, present_value) in changed_slots {
        if slot == current_epoch_slot() {
            if current_epoch_write
                .replace((original_value, present_value))
                .is_some()
            {
                return Err(EpochBoundaryEffectError::DuplicateChangedSlot(slot));
            }
        } else if slot == next_epoch_block_slot() {
            if next_epoch_block_write
                .replace((original_value, present_value))
                .is_some()
            {
                return Err(EpochBoundaryEffectError::DuplicateChangedSlot(slot));
            }
        } else if epoch_start_block_write
            .replace((slot, original_value, present_value))
            .is_some()
        {
            return Err(EpochBoundaryEffectError::DuplicateChangedSlot(slot));
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

    let epoch_start_block = decode_epoch_start_block_storage_value(epoch_start_value)
        .ok_or(EpochBoundaryEffectError::InvalidEpochStartBlockEncoding)?;
    let next_epoch_block_value = match next_epoch_block_write {
        Some((_original, present)) => {
            decode_u64_storage_value(present)
                .ok_or(EpochBoundaryEffectError::InvalidNextEpochBlockValue)?;
            present
        }
        None => U256::from(epoch_start_block),
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

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256};
    use reth_evm::revm::state::{Account, AccountInfo, EvmState, EvmStorageSlot};

    use super::*;

    fn changed_slot(original: u64, present: U256) -> EvmStorageSlot {
        EvmStorageSlot::new_changed(U256::from(original), present, 0)
    }

    #[test]
    fn extracts_storage_ready_epoch_boundary_effect() {
        let current_epoch = 1_u64;
        let original_next_epoch_block = 14_u64;
        let next_epoch_block = 24_u64;
        let start_slot = epoch_start_block_slot(current_epoch);
        let start_value = U256::from(original_next_epoch_block + 1);

        let mut account = Account::default();
        account.storage.insert(
            current_epoch_slot(),
            changed_slot(0, U256::from(current_epoch)),
        );
        account.storage.insert(
            next_epoch_block_slot(),
            changed_slot(original_next_epoch_block, U256::from(next_epoch_block)),
        );
        account
            .storage
            .insert(start_slot, changed_slot(0, start_value));

        let mut outcome_state = EvmState::default();
        outcome_state.insert(EPOCH_PRECOMPILE_ADDRESS, account);

        let effect = extract_epoch_boundary_effect(&outcome_state).expect("extract effect");
        assert_eq!(
            effect.writes,
            [
                EpochBoundaryStorageWrite {
                    slot: current_epoch_slot(),
                    value: U256::from(current_epoch),
                },
                EpochBoundaryStorageWrite {
                    slot: next_epoch_block_slot(),
                    value: U256::from(next_epoch_block),
                },
                EpochBoundaryStorageWrite {
                    slot: start_slot,
                    value: start_value,
                },
            ]
        );
    }

    #[test]
    fn synthesizes_next_epoch_block_write_from_start_block_when_missing() {
        let current_epoch = 2_u64;
        let start_slot = epoch_start_block_slot(current_epoch);
        let start_value = U256::from(3_u64);

        let mut account = Account::default();
        account.storage.insert(
            current_epoch_slot(),
            changed_slot(1, U256::from(current_epoch)),
        );
        account
            .storage
            .insert(start_slot, changed_slot(0, start_value));

        let mut outcome_state = EvmState::default();
        outcome_state.insert(EPOCH_PRECOMPILE_ADDRESS, account);

        let effect = extract_epoch_boundary_effect(&outcome_state).expect("extract effect");
        assert_eq!(
            effect.writes[1],
            EpochBoundaryStorageWrite {
                slot: next_epoch_block_slot(),
                value: U256::from(2_u64),
            }
        );
    }

    #[test]
    fn rejects_account_info_replay_dependency() {
        let mut account = Account::default();
        account.info = AccountInfo::from_balance(U256::from(1_u64));
        account.original_info = Box::new(AccountInfo::default());
        account
            .storage
            .insert(current_epoch_slot(), changed_slot(0, U256::from(1_u64)));
        account
            .storage
            .insert(next_epoch_block_slot(), changed_slot(1, U256::from(12_u64)));
        account.storage.insert(
            epoch_start_block_slot(1),
            changed_slot(0, U256::from(2_u64)),
        );

        let mut outcome_state = EvmState::default();
        outcome_state.insert(EPOCH_PRECOMPILE_ADDRESS, account);

        let err = extract_epoch_boundary_effect(&outcome_state).expect_err("info replay rejected");
        assert_eq!(err, EpochBoundaryEffectError::AccountInfoReplayRequired);
    }

    #[test]
    fn rejects_unexpected_changed_account() {
        let mut account = Account::default();
        account
            .storage
            .insert(current_epoch_slot(), changed_slot(0, U256::from(1_u64)));

        let mut outcome_state = EvmState::default();
        outcome_state.insert(Address::with_last_byte(7), account);

        let err =
            extract_epoch_boundary_effect(&outcome_state).expect_err("unexpected account rejected");
        assert_eq!(err, EpochBoundaryEffectError::UnexpectedChangedAccount);
    }

    #[test]
    fn rejects_missing_epoch_start_block_write_when_encoding_would_not_change_storage() {
        let mut account = Account::default();
        account
            .storage
            .insert(current_epoch_slot(), changed_slot(0, U256::from(1_u64)));
        account.storage.insert(
            next_epoch_block_slot(),
            changed_slot(10, U256::from(20_u64)),
        );
        account
            .storage
            .insert(epoch_start_block_slot(1), changed_slot(0, U256::ZERO));

        let mut outcome_state = EvmState::default();
        outcome_state.insert(EPOCH_PRECOMPILE_ADDRESS, account);

        let err = extract_epoch_boundary_effect(&outcome_state)
            .expect_err("missing changed epoch start slot rejected");
        assert_eq!(err, EpochBoundaryEffectError::MissingEpochStartBlockWrite);
    }

    #[test]
    fn rejects_already_initialized_epoch_start_block_slot() {
        let mut account = Account::default();
        account
            .storage
            .insert(current_epoch_slot(), changed_slot(0, U256::from(1_u64)));
        account.storage.insert(
            next_epoch_block_slot(),
            changed_slot(10, U256::from(20_u64)),
        );
        account.storage.insert(
            epoch_start_block_slot(1),
            changed_slot(1, U256::from(2_u64)),
        );

        let mut outcome_state = EvmState::default();
        outcome_state.insert(EPOCH_PRECOMPILE_ADDRESS, account);

        let err = extract_epoch_boundary_effect(&outcome_state)
            .expect_err("already initialized start slot rejected");
        assert_eq!(
            err,
            EpochBoundaryEffectError::EpochStartBlockAlreadyInitialized
        );
    }
}
