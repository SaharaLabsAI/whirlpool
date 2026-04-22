use alloy_primitives::{Address, U256};
use evm_precompiles::{
    community_pool_last_processed_epoch_slot, community_pool_locked_remaining_slot,
    community_pool_unlock_amount_per_cycle_slot, community_pool_unlock_every_epochs_slot,
    COMMUNITY_POOL_ADDRESS,
};

use crate::{error::EvmAppError, traits::StateDb};

pub fn credit_account_balance<DB>(
    db: &mut DB,
    address: Address,
    amount: U256,
) -> Result<(), EvmAppError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Into<EvmAppError>,
{
    if amount.is_zero() {
        return Ok(());
    }

    let mut info = db
        .get_account(address)
        .map_err(Into::into)?
        .unwrap_or_default();
    info.balance += amount;
    insert_account_preserving_community_pool_unlock_storage(db, address, info)
}

pub fn insert_account_preserving_community_pool_unlock_storage<DB>(
    db: &mut DB,
    address: Address,
    info: revm::state::AccountInfo,
) -> Result<(), EvmAppError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Into<EvmAppError>,
{
    if address != COMMUNITY_POOL_ADDRESS {
        return db.insert_account(address, info).map_err(Into::into);
    }

    let unlock_every_epochs_slot = community_pool_unlock_every_epochs_slot();
    let unlock_amount_per_cycle_slot = community_pool_unlock_amount_per_cycle_slot();
    let locked_remaining_slot = community_pool_locked_remaining_slot();
    let last_processed_epoch_slot = community_pool_last_processed_epoch_slot();

    let unlock_every_epochs = db
        .get_storage(COMMUNITY_POOL_ADDRESS, unlock_every_epochs_slot)
        .map_err(Into::into)?;
    let unlock_amount_per_cycle = db
        .get_storage(COMMUNITY_POOL_ADDRESS, unlock_amount_per_cycle_slot)
        .map_err(Into::into)?;
    let locked_remaining = db
        .get_storage(COMMUNITY_POOL_ADDRESS, locked_remaining_slot)
        .map_err(Into::into)?;
    let last_processed_epoch = db
        .get_storage(COMMUNITY_POOL_ADDRESS, last_processed_epoch_slot)
        .map_err(Into::into)?;

    db.insert_account(address, info).map_err(Into::into)?;
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        unlock_every_epochs_slot,
        unlock_every_epochs,
    )
    .map_err(Into::into)?;
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        unlock_amount_per_cycle_slot,
        unlock_amount_per_cycle,
    )
    .map_err(Into::into)?;
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        locked_remaining_slot,
        locked_remaining,
    )
    .map_err(Into::into)?;
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        last_processed_epoch_slot,
        last_processed_epoch,
    )
    .map_err(Into::into)
}

pub fn transfer_account_balance<DB>(
    db: &mut DB,
    from: Address,
    to: Address,
    amount: U256,
) -> Result<(), EvmAppError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Into<EvmAppError>,
{
    if amount.is_zero() {
        return Ok(());
    }

    let mut from_info = db
        .get_account(from)
        .map_err(Into::into)?
        .unwrap_or_default();
    if from_info.balance < amount {
        return Err(EvmAppError::Execution(format!(
            "insufficient balance for unlock transfer from {from}: balance={}, required={amount}",
            from_info.balance
        )));
    }
    from_info.balance -= amount;
    insert_account_preserving_community_pool_unlock_storage(db, from, from_info)?;

    let mut to_info = db.get_account(to).map_err(Into::into)?.unwrap_or_default();
    to_info.balance = to_info
        .balance
        .checked_add(amount)
        .ok_or_else(|| EvmAppError::Execution("fee-pool balance overflow".into()))?;
    insert_account_preserving_community_pool_unlock_storage(db, to, to_info)
}
