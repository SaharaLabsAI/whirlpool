use std::fmt::Display;

use alloy_primitives::{Address, U256};
use reth_evm::revm::state::AccountInfo;
use state::StateDb;

use crate::{current_epoch_slot, EPOCH_PRECOMPILE_ADDRESS, FEE_POOL_PRECOMPILE_ADDRESS};
use crate::fee_pool::credit_fee_pool_claim;

use super::{
    build_post_block_accounting_effect, community_pool_last_processed_epoch_slot,
    community_pool_locked_remaining_slot, community_pool_unlock_amount_per_cycle_slot,
    community_pool_unlock_every_epochs_slot, CommunityPoolUnlockEffect,
    CommunityPoolUnlockState, PostBlockAccountingEffectError, PostBlockAccountingInputs,
    PostBlockAccountingOutcome, COMMUNITY_POOL_ADDRESS,
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PostBlockAccountingRuntimeError {
    #[error("state access error: {0}")]
    StateAccess(String),
    #[error("{0}")]
    InvalidStoredValue(String),
    #[error("{0}")]
    Execution(String),
}

impl From<PostBlockAccountingEffectError> for PostBlockAccountingRuntimeError {
    fn from(err: PostBlockAccountingEffectError) -> Self {
        match err {
            PostBlockAccountingEffectError::InvalidStoredValue(message) => {
                PostBlockAccountingRuntimeError::InvalidStoredValue(message)
            }
            PostBlockAccountingEffectError::Execution(message) => {
                PostBlockAccountingRuntimeError::Execution(message)
            }
        }
    }
}

pub fn apply_post_block_accounting<DB>(
    db: &mut DB,
    inputs: &PostBlockAccountingInputs,
) -> Result<PostBlockAccountingOutcome, PostBlockAccountingRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    // Callers must invoke this only after any required epoch-boundary effect has
    // already been applied, and `base_fee_per_gas` must be the canonical
    // protocol-derived next-block base fee for the block being accounted.
    let current_epoch = load_u64_storage_value(
        db,
        EPOCH_PRECOMPILE_ADDRESS,
        current_epoch_slot(),
        "epoch currentEpoch",
    )?;
    let unlock_state = load_community_pool_unlock_state(db)?;
    let effect = build_post_block_accounting_effect(inputs, current_epoch, unlock_state)?;

    if let Some(ref unlock_effect) = effect.community_pool_unlock {
        apply_community_pool_unlock_effect(db, unlock_effect)?;
    }
    credit_account_balance(db, COMMUNITY_POOL_ADDRESS, effect.burned_fees)?;
    if let Some(ref claim) = effect.priority_fee_claim {
        credit_fee_pool_claim(db, claim)?;
    }

    Ok(PostBlockAccountingOutcome {
        current_epoch,
        effect,
    })
}

fn load_community_pool_unlock_state<DB>(
    db: &DB,
) -> Result<CommunityPoolUnlockState, PostBlockAccountingRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    Ok(CommunityPoolUnlockState {
        unlock_every_epochs: load_u64_storage_value(
            db,
            COMMUNITY_POOL_ADDRESS,
            community_pool_unlock_every_epochs_slot(),
            "community-pool unlockEveryEpochs",
        )?,
        unlock_amount_per_cycle: db
            .get_storage(
                COMMUNITY_POOL_ADDRESS,
                community_pool_unlock_amount_per_cycle_slot(),
            )
            .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?,
        locked_remaining: db
            .get_storage(
                COMMUNITY_POOL_ADDRESS,
                community_pool_locked_remaining_slot(),
            )
            .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?,
        last_processed_epoch: load_u64_storage_value(
            db,
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot(),
            "community-pool lastProcessedEpoch",
        )?,
    })
}

fn load_u64_storage_value<DB>(
    db: &DB,
    address: Address,
    slot: U256,
    field: &str,
) -> Result<u64, PostBlockAccountingRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    let raw = db
        .get_storage(address, slot)
        .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?;
    u64::try_from(raw).map_err(|_| {
        PostBlockAccountingRuntimeError::InvalidStoredValue(format!(
            "{field} storage does not fit into u64: {raw}"
        ))
    })
}

fn apply_community_pool_unlock_effect<DB>(
    db: &mut DB,
    effect: &CommunityPoolUnlockEffect,
) -> Result<(), PostBlockAccountingRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    if !effect.unlock_tranche.is_zero() {
        transfer_account_balance(
            db,
            COMMUNITY_POOL_ADDRESS,
            FEE_POOL_PRECOMPILE_ADDRESS,
            effect.unlock_tranche,
        )?;
        for claim in &effect.validator_claims {
            credit_fee_pool_claim(db, claim)?;
        }
    }

    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_locked_remaining_slot(),
        effect.next_locked_remaining,
    )
    .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?;
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_last_processed_epoch_slot(),
        U256::from(effect.last_processed_epoch),
    )
    .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))
}

fn credit_account_balance<DB>(
    db: &mut DB,
    address: Address,
    amount: U256,
) -> Result<(), PostBlockAccountingRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    if amount.is_zero() {
        return Ok(());
    }

    let mut info = db
        .get_account(address)
        .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?
        .unwrap_or_default();
    info.balance += amount;
    insert_account_preserving_community_pool_state(db, address, info)
}

fn transfer_account_balance<DB>(
    db: &mut DB,
    from: Address,
    to: Address,
    amount: U256,
) -> Result<(), PostBlockAccountingRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    if amount.is_zero() {
        return Ok(());
    }

    let mut from_info = db
        .get_account(from)
        .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?
        .unwrap_or_default();
    if from_info.balance < amount {
        return Err(PostBlockAccountingRuntimeError::Execution(format!(
            "insufficient balance for unlock transfer from {from}: balance={}, required={amount}",
            from_info.balance
        )));
    }
    from_info.balance -= amount;
    insert_account_preserving_community_pool_state(db, from, from_info)?;

    let mut to_info = db
        .get_account(to)
        .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?
        .unwrap_or_default();
    to_info.balance = to_info.balance.checked_add(amount).ok_or_else(|| {
        PostBlockAccountingRuntimeError::Execution("fee-pool balance overflow".into())
    })?;
    insert_account_preserving_community_pool_state(db, to, to_info)
}

fn insert_account_preserving_community_pool_state<DB>(
    db: &mut DB,
    address: Address,
    info: AccountInfo,
) -> Result<(), PostBlockAccountingRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    if address != COMMUNITY_POOL_ADDRESS {
        return db
            .insert_account(address, info)
            .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()));
    }

    let unlock_every_epochs = db
        .get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_unlock_every_epochs_slot(),
        )
        .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?;
    let unlock_amount_per_cycle = db
        .get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_unlock_amount_per_cycle_slot(),
        )
        .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?;
    let locked_remaining = db
        .get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot(),
        )
        .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?;
    let last_processed_epoch = db
        .get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot(),
        )
        .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?;

    db.insert_account(address, info)
        .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?;
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_unlock_every_epochs_slot(),
        unlock_every_epochs,
    )
    .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?;
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_unlock_amount_per_cycle_slot(),
        unlock_amount_per_cycle,
    )
    .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?;
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_locked_remaining_slot(),
        locked_remaining,
    )
    .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?;
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_last_processed_epoch_slot(),
        last_processed_epoch,
    )
    .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Address;
    use app_evm_state::InMemoryStateDb;
    use validators::ValidatorEntry;

    use super::*;
    use crate::{
        claimable_balance_slot, community_pool_last_processed_epoch_slot,
        community_pool_locked_remaining_slot, community_pool_unlock_amount_per_cycle_slot,
        community_pool_unlock_every_epochs_slot, current_epoch_slot,
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
}
