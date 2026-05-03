use std::fmt::Display;

use alloy_primitives::{Address, U256};
use reth_evm::revm::state::AccountInfo;
use state::StateDb;

use crate::fee_claim_writer::credit_fee_pool_claim;
use crate::{current_epoch_slot, EPOCH_PRECOMPILE_ADDRESS, FEE_POOL_PRECOMPILE_ADDRESS};

use crate::community_pool::{
    build_post_block_accounting_effect, community_pool_last_processed_epoch_slot,
    community_pool_locked_remaining_slot, community_pool_unlock_amount_per_cycle_slot,
    community_pool_unlock_every_epochs_slot, CommunityPoolUnlockEffect, CommunityPoolUnlockState,
    PostBlockAccountingEffectError, PostBlockAccountingInputs, PostBlockAccountingOutcome,
    COMMUNITY_POOL_ADDRESS,
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
mod tests;
