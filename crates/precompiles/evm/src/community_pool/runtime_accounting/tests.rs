use alloy_primitives::{Address, U256};
use app_evm_state::InMemoryStateDb;
use reth_evm::revm::state::AccountInfo;
use validators_reader::ValidatorEntry;

use crate::community_pool::runtime_accounting::{
    apply_post_block_accounting, PostBlockAccountingInputs,
};
use crate::{
    claimable_balance_slot, community_pool_last_processed_epoch_slot,
    community_pool_locked_remaining_slot, community_pool_unlock_amount_per_cycle_slot,
    community_pool_unlock_every_epochs_slot, current_epoch_slot, COMMUNITY_POOL_ADDRESS,
    EPOCH_PRECOMPILE_ADDRESS, FEE_POOL_PRECOMPILE_ADDRESS,
};

fn account_balance(db: &InMemoryStateDb, address: Address) -> U256 {
    db.get_account(address).unwrap_or_default().balance
}

fn storage_value(db: &InMemoryStateDb, address: Address, slot: U256) -> U256 {
    db.get_storage(address, slot)
}

fn seed_unlock_state(
    db: &mut InMemoryStateDb,
    current_epoch: u64,
    unlock_every_epochs: u64,
    unlock_amount_per_cycle: U256,
    locked_remaining: U256,
    community_pool_balance: U256,
) {
    db.insert_account(
        COMMUNITY_POOL_ADDRESS,
        AccountInfo {
            balance: community_pool_balance,
            nonce: 0,
            ..Default::default()
        },
    );
    db.insert_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        current_epoch_slot(),
        U256::from(current_epoch),
    );
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_unlock_every_epochs_slot(),
        U256::from(unlock_every_epochs),
    );
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_unlock_amount_per_cycle_slot(),
        unlock_amount_per_cycle,
    );
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_locked_remaining_slot(),
        locked_remaining,
    );
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_last_processed_epoch_slot(),
        U256::ZERO,
    );
}

fn sample_inputs(validators: Vec<ValidatorEntry>) -> PostBlockAccountingInputs {
    PostBlockAccountingInputs {
        boundary_required: true,
        gas_used: 0,
        base_fee_per_gas: 1,
        priority_fees: U256::from(7_u64),
        claim_recipient: Address::repeat_byte(0xaa),
        simplex_validators: validators,
    }
}

#[test]
fn apply_post_block_accounting_preserves_unlock_slots_when_crediting_burned_fees() {
    let mut db = InMemoryStateDb::new();
    seed_unlock_state(
        &mut db,
        1,
        0,
        U256::ZERO,
        U256::from(25_u64),
        U256::from(25_u64),
    );

    let inputs = PostBlockAccountingInputs {
        boundary_required: false,
        gas_used: 3,
        base_fee_per_gas: 2,
        priority_fees: U256::ZERO,
        claim_recipient: Address::repeat_byte(0xaa),
        simplex_validators: vec![],
    };

    let outcome =
        apply_post_block_accounting(&mut db, &inputs).expect("apply post-block accounting");
    assert_eq!(outcome.current_epoch, 1);
    assert_eq!(
        account_balance(&db, COMMUNITY_POOL_ADDRESS),
        U256::from(31_u64)
    );
    assert_eq!(
        storage_value(
            &db,
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot(),
        ),
        U256::from(25_u64)
    );
    assert_eq!(
        storage_value(
            &db,
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot(),
        ),
        U256::ZERO
    );
}

#[test]
fn apply_post_block_accounting_updates_priority_fee_claim_slot() {
    let mut db = InMemoryStateDb::new();
    seed_unlock_state(&mut db, 1, 0, U256::ZERO, U256::ZERO, U256::ZERO);
    let claim_recipient = Address::repeat_byte(0xaa);
    let inputs = PostBlockAccountingInputs {
        claim_recipient,
        ..sample_inputs(vec![])
    };

    apply_post_block_accounting(&mut db, &inputs).expect("apply post-block accounting");

    assert_eq!(
        storage_value(
            &db,
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(claim_recipient),
        ),
        U256::from(7_u64)
    );
}

#[test]
fn apply_post_block_accounting_is_idempotent_for_same_epoch_unlock() {
    let validators = vec![
        ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: Address::repeat_byte(0x11),
        },
        ValidatorEntry {
            consensus_pubkey: [0x22; 32],
            ethereum_address: Address::repeat_byte(0x22),
        },
    ];
    let mut db = InMemoryStateDb::new();
    seed_unlock_state(
        &mut db,
        2,
        2,
        U256::from(10_u64),
        U256::from(25_u64),
        U256::from(25_u64),
    );

    let inputs = sample_inputs(validators.clone());
    apply_post_block_accounting(&mut db, &inputs).expect("first apply");
    let community_pool_before = account_balance(&db, COMMUNITY_POOL_ADDRESS);
    let fee_pool_before = account_balance(&db, FEE_POOL_PRECOMPILE_ADDRESS);
    let locked_before = storage_value(
        &db,
        COMMUNITY_POOL_ADDRESS,
        community_pool_locked_remaining_slot(),
    );
    let claim0_before = storage_value(
        &db,
        FEE_POOL_PRECOMPILE_ADDRESS,
        claimable_balance_slot(validators[0].ethereum_address),
    );
    let claim1_before = storage_value(
        &db,
        FEE_POOL_PRECOMPILE_ADDRESS,
        claimable_balance_slot(validators[1].ethereum_address),
    );
    let proposer_claim_before = storage_value(
        &db,
        FEE_POOL_PRECOMPILE_ADDRESS,
        claimable_balance_slot(inputs.claim_recipient),
    );

    apply_post_block_accounting(&mut db, &inputs).expect("second apply");

    assert_eq!(
        account_balance(&db, COMMUNITY_POOL_ADDRESS),
        community_pool_before
    );
    assert_eq!(
        account_balance(&db, FEE_POOL_PRECOMPILE_ADDRESS),
        fee_pool_before
    );
    assert_eq!(
        storage_value(
            &db,
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot(),
        ),
        locked_before
    );
    assert_eq!(
        storage_value(
            &db,
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validators[0].ethereum_address),
        ),
        claim0_before
    );
    assert_eq!(
        storage_value(
            &db,
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validators[1].ethereum_address),
        ),
        claim1_before
    );
    assert_eq!(
        storage_value(
            &db,
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(inputs.claim_recipient),
        ),
        proposer_claim_before + U256::from(7_u64)
    );
}
