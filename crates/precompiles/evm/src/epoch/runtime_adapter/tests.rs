use alloy_primitives::{Address, U256};
use app_evm_state::InMemoryStateDb;
use reth_evm::revm::{
    database::{CacheDB, EmptyDB},
    state::{Account, EvmState, EvmStorageSlot},
    DatabaseRef,
};
use reth_evm::{EvmEnv, EvmFactory};

use crate::epoch::runtime_adapter::*;
use crate::epoch::{
    current_epoch_slot, epoch_blocks_slot, epoch_start_block_slot, EpochBoundaryStorageWrite,
};
use crate::WhirlpoolEvmFactory;

fn storage_value(db: &InMemoryStateDb, slot: U256) -> U256 {
    db.get_storage(EPOCH_PRECOMPILE_ADDRESS, slot)
}

fn seeded_boundary_db() -> CacheDB<EmptyDB> {
    let mut db = CacheDB::<EmptyDB>::default();
    db.insert_account_info(EPOCH_PRECOMPILE_ADDRESS, Default::default());
    db.insert_account_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot(), U256::ZERO)
        .expect("seed current epoch");
    db.insert_account_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        epoch_blocks_slot(),
        U256::from(10_u64),
    )
    .expect("seed epoch blocks");
    db.insert_account_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        next_epoch_block_slot(),
        U256::from(5_u64),
    )
    .expect("seed next epoch block");
    db.insert_account_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        epoch_start_block_slot(0),
        U256::from(1_u64),
    )
    .expect("seed epoch zero start");
    db
}

fn changed_slot(original: u64, present: U256) -> EvmStorageSlot {
    EvmStorageSlot::new_changed(U256::from(original), present, 0)
}

#[test]
fn load_epoch_boundary_state_from_statedb() {
    let mut db = InMemoryStateDb::new();
    db.insert_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        next_epoch_block_slot(),
        U256::from(42_u64),
    );

    let state = load_epoch_boundary_state(&db).expect("load boundary state");

    assert_eq!(state.next_epoch_block, 42);
}

#[test]
fn apply_epoch_boundary_effect_via_statedb() {
    let mut db = InMemoryStateDb::new();
    db.insert_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot(), U256::ZERO);
    db.insert_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        next_epoch_block_slot(),
        U256::from(10_u64),
    );

    let effect = EpochBoundaryEffect {
        writes: [
            EpochBoundaryStorageWrite {
                slot: current_epoch_slot(),
                value: U256::from(1_u64),
            },
            EpochBoundaryStorageWrite {
                slot: next_epoch_block_slot(),
                value: U256::from(20_u64),
            },
            EpochBoundaryStorageWrite {
                slot: epoch_start_block_slot(1),
                value: U256::from(11_u64),
            },
        ],
    };

    apply_epoch_boundary_effect(&mut db, &effect).expect("apply effect");

    assert_eq!(storage_value(&db, current_epoch_slot()), U256::from(1_u64));
    assert_eq!(
        storage_value(&db, next_epoch_block_slot()),
        U256::from(20_u64)
    );
    assert_eq!(
        storage_value(&db, epoch_start_block_slot(1)),
        U256::from(11_u64)
    );
}

#[test]
fn execute_epoch_boundary_system_call_if_required_commits_after_validation() {
    let db = seeded_boundary_db();
    let evm_env = EvmEnv::default().with_block_number(U256::from(5_u64));
    let mut evm = WhirlpoolEvmFactory.create_evm(db, evm_env);

    let effect = execute_epoch_boundary_system_call_if_required(&mut evm, true)
        .expect("first boundary system call should succeed")
        .expect("boundary effect should exist");
    assert_eq!(effect.writes[0].slot, current_epoch_slot());
    assert_eq!(effect.writes[0].value, U256::from(1_u64));
    assert_eq!(effect.writes[1].slot, next_epoch_block_slot());
    assert_eq!(effect.writes[1].value, U256::from(15_u64));

    let second = execute_epoch_boundary_system_call_if_required(&mut evm, true)
        .expect_err("second call should observe committed in-memory state");
    assert_eq!(second, EpochBoundaryRuntimeError::SystemCallUnsuccessful);

    let db = evm.db_mut();
    assert_eq!(
        db.storage_ref(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot())
            .expect("read current epoch"),
        U256::from(1_u64)
    );
    assert_eq!(
        db.storage_ref(EPOCH_PRECOMPILE_ADDRESS, next_epoch_block_slot())
            .expect("read next epoch block"),
        U256::from(15_u64)
    );
    assert_eq!(
        db.storage_ref(EPOCH_PRECOMPILE_ADDRESS, epoch_start_block_slot(2))
            .expect("read uninitialized epoch two start"),
        U256::ZERO
    );
}

#[test]
fn execute_epoch_boundary_system_call_if_required_skips_noop_without_commit() {
    let db = seeded_boundary_db();
    let evm_env = EvmEnv::default().with_block_number(U256::from(4_u64));
    let mut evm = WhirlpoolEvmFactory.create_evm(db, evm_env);

    let effect = execute_epoch_boundary_system_call_if_required(&mut evm, false)
        .expect("no-op boundary check should succeed");

    assert_eq!(effect, None);
    let db = evm.db_mut();
    assert_eq!(
        db.storage_ref(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot())
            .expect("read current epoch"),
        U256::ZERO
    );
    assert_eq!(
        db.storage_ref(EPOCH_PRECOMPILE_ADDRESS, next_epoch_block_slot())
            .expect("read next epoch block"),
        U256::from(5_u64)
    );
    assert_eq!(
        db.storage_ref(EPOCH_PRECOMPILE_ADDRESS, epoch_start_block_slot(1))
            .expect("read uninitialized epoch one start"),
        U256::ZERO
    );
}

#[test]
fn invalid_epoch_boundary_effect_is_not_committed() {
    let unexpected_account = Address::with_last_byte(7);
    let invalid_slot = current_epoch_slot();
    let invalid_value = U256::from(1_u64);
    let mut invalid_account = Account::default();
    invalid_account.mark_touch();
    invalid_account
        .storage
        .insert(invalid_slot, changed_slot(0, invalid_value));
    let mut outcome_state = EvmState::default();
    outcome_state.insert(unexpected_account, invalid_account);

    let mut db = seeded_boundary_db();
    let err = commit_validated_epoch_boundary_effect(&mut db, outcome_state)
        .expect_err("invalid effect must fail before commit");

    assert!(matches!(
        err,
        EpochBoundaryRuntimeError::EffectExtraction(_)
    ));
    assert_eq!(
        db.storage_ref(unexpected_account, invalid_slot)
            .expect("read unexpected account slot"),
        U256::ZERO
    );
}
