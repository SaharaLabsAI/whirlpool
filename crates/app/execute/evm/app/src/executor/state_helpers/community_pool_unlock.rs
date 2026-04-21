use alloy_primitives::{Address, U256};
use evm_precompiles::{
    community_pool_last_processed_epoch_slot, community_pool_locked_remaining_slot,
    community_pool_unlock_amount_per_cycle_slot, community_pool_unlock_every_epochs_slot,
    current_epoch_slot, COMMUNITY_POOL_ADDRESS, EPOCH_PRECOMPILE_ADDRESS,
    FEE_POOL_PRECOMPILE_ADDRESS,
};
use validators::ValidatorEntry;

use crate::{error::EvmAppError, traits::StateProvider};

use super::account_balances::transfer_account_balance;
use super::fee_accounting::credit_fee_pool_claim;

pub fn load_u64_storage_value<DB>(
    db: &DB,
    address: Address,
    slot: U256,
    field: &str,
) -> Result<u64, EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    let raw = db.get_storage(address, slot).map_err(Into::into)?;
    u64::try_from(raw).map_err(|_| {
        EvmAppError::InvalidBlock(format!("{field} storage does not fit into u64: {raw}"))
    })
}

pub fn maybe_apply_community_pool_unlock<DB>(
    db: &mut DB,
    boundary_required: bool,
    simplex_validators: &[ValidatorEntry],
) -> Result<(), EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    if !boundary_required {
        return Ok(());
    }

    let unlock_every_epochs_slot = community_pool_unlock_every_epochs_slot();
    let unlock_amount_per_cycle_slot = community_pool_unlock_amount_per_cycle_slot();
    let locked_remaining_slot = community_pool_locked_remaining_slot();
    let last_processed_epoch_slot = community_pool_last_processed_epoch_slot();

    let unlock_every_epochs = load_u64_storage_value(
        db,
        COMMUNITY_POOL_ADDRESS,
        unlock_every_epochs_slot,
        "community-pool unlockEveryEpochs",
    )?;
    let unlock_amount_per_cycle = db
        .get_storage(COMMUNITY_POOL_ADDRESS, unlock_amount_per_cycle_slot)
        .map_err(Into::into)?;

    let unlock_enabled = unlock_every_epochs > 0 && !unlock_amount_per_cycle.is_zero();
    if !unlock_enabled {
        return Ok(());
    }

    if simplex_validators.is_empty() {
        return Err(EvmAppError::Execution(
            "community-pool unlock schedule enabled but simplex validators are empty".into(),
        ));
    }

    let current_epoch = load_u64_storage_value(
        db,
        EPOCH_PRECOMPILE_ADDRESS,
        current_epoch_slot(),
        "epoch currentEpoch",
    )?;
    if current_epoch == 0 || current_epoch % unlock_every_epochs != 0 {
        return Ok(());
    }

    let last_processed_epoch = load_u64_storage_value(
        db,
        COMMUNITY_POOL_ADDRESS,
        last_processed_epoch_slot,
        "community-pool lastProcessedEpoch",
    )?;
    if last_processed_epoch > current_epoch {
        return Err(EvmAppError::InvalidBlock(format!(
            "community-pool lastProcessedEpoch {last_processed_epoch} exceeds current epoch {current_epoch}"
        )));
    }
    if last_processed_epoch == current_epoch {
        return Ok(());
    }

    let locked_remaining = db
        .get_storage(COMMUNITY_POOL_ADDRESS, locked_remaining_slot)
        .map_err(Into::into)?;
    if locked_remaining.is_zero() {
        db.insert_storage(
            COMMUNITY_POOL_ADDRESS,
            last_processed_epoch_slot,
            U256::from(current_epoch),
        )
        .map_err(Into::into)?;
        return Ok(());
    }

    let unlock_tranche = unlock_amount_per_cycle.min(locked_remaining);
    if unlock_tranche.is_zero() {
        db.insert_storage(
            COMMUNITY_POOL_ADDRESS,
            last_processed_epoch_slot,
            U256::from(current_epoch),
        )
        .map_err(Into::into)?;
        return Ok(());
    }

    transfer_account_balance(
        db,
        COMMUNITY_POOL_ADDRESS,
        FEE_POOL_PRECOMPILE_ADDRESS,
        unlock_tranche,
    )?;

    let validator_count = U256::from(
        u64::try_from(simplex_validators.len())
            .map_err(|_| EvmAppError::Execution("validator count does not fit into u64".into()))?,
    );
    let base_share = unlock_tranche / validator_count;
    let remainder_u64 = u64::try_from(unlock_tranche % validator_count).map_err(|_| {
        EvmAppError::Execution("community-pool unlock remainder does not fit into u64".into())
    })?;
    let remainder = usize::try_from(remainder_u64).map_err(|_| {
        EvmAppError::Execution("community-pool unlock remainder does not fit into usize".into())
    })?;

    let mut total_credited = U256::ZERO;
    for (index, validator) in simplex_validators.iter().enumerate() {
        let extra = if index < remainder {
            U256::from(1_u64)
        } else {
            U256::ZERO
        };
        let share = base_share
            .checked_add(extra)
            .ok_or_else(|| EvmAppError::Execution("community-pool share overflow".into()))?;
        credit_fee_pool_claim(db, validator.ethereum_address, share)?;
        total_credited = total_credited
            .checked_add(share)
            .ok_or_else(|| EvmAppError::Execution("community-pool total credit overflow".into()))?;
    }

    if total_credited != unlock_tranche {
        return Err(EvmAppError::Execution(format!(
            "community-pool unlock accounting mismatch: credited {total_credited}, tranche {unlock_tranche}"
        )));
    }

    let next_locked_remaining = locked_remaining
        .checked_sub(unlock_tranche)
        .ok_or_else(|| EvmAppError::Execution("community-pool remaining underflow".into()))?;
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        locked_remaining_slot,
        next_locked_remaining,
    )
    .map_err(Into::into)?;
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        last_processed_epoch_slot,
        U256::from(current_epoch),
    )
    .map_err(Into::into)
}
