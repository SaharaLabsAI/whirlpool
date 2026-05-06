use alloy_primitives::{Address, U256};
use reth_evm::revm::state::{Account, AccountInfo, EvmState, EvmStorageSlot};

use crate::epoch::boundary_effect::*;
use crate::epoch::{current_epoch_slot, epoch_start_block_slot, next_epoch_block_slot};

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
fn rejects_missing_next_epoch_block_write() {
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

    let err = extract_epoch_boundary_effect(&outcome_state)
        .expect_err("missing next epoch block write rejected");
    assert_eq!(err, EpochBoundaryEffectError::MissingNextEpochBlockWrite);
}

#[test]
fn synthesizes_next_epoch_block_from_loaded_runtime_context_when_write_is_missing() {
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
    account.storage.insert(
        epoch_blocks_slot(),
        EvmStorageSlot::new(U256::from(1_u64), 0),
    );
    account.storage.insert(
        next_epoch_block_slot(),
        EvmStorageSlot::new(U256::from(2_u64), 0),
    );

    let mut outcome_state = EvmState::default();
    outcome_state.insert(EPOCH_PRECOMPILE_ADDRESS, account);

    let effect = extract_epoch_boundary_effect(&outcome_state)
        .expect("loaded runtime context should synthesize next epoch block");
    assert_eq!(
        effect.writes[1],
        EpochBoundaryStorageWrite {
            slot: next_epoch_block_slot(),
            value: U256::from(3_u64),
        }
    );
}

#[test]
fn rejects_account_info_replay_dependency() {
    let mut account = Account {
        info: AccountInfo::from_balance(U256::from(1_u64)),
        original_info: Box::new(AccountInfo::default()),
        ..Default::default()
    };
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
