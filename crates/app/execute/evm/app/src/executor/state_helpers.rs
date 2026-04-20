use super::*;

pub fn credit_account_balance<DB>(
    db: &mut DB,
    address: Address,
    amount: U256,
) -> Result<(), EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
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
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
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

pub fn credit_burned_fees<DB>(
    db: &mut DB,
    gas_used: u64,
    base_fee_per_gas: u64,
) -> Result<(), EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    let burned_amount = U256::from(gas_used) * U256::from(base_fee_per_gas);
    credit_account_balance(db, COMMUNITY_POOL_ADDRESS, burned_amount)
}

pub fn credit_fee_pool_claim<DB>(
    db: &mut DB,
    recipient: Address,
    amount: U256,
) -> Result<(), EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    if amount.is_zero() {
        return Ok(());
    }

    let slot = claimable_balance_slot(recipient);
    let current = db
        .get_storage(FEE_POOL_PRECOMPILE_ADDRESS, slot)
        .map_err(Into::into)?;
    let next = current
        .checked_add(amount)
        .ok_or_else(|| EvmAppError::Execution("fee-pool claim ledger overflow".into()))?;

    db.insert_storage(FEE_POOL_PRECOMPILE_ADDRESS, slot, next)
        .map_err(Into::into)
}

pub fn transfer_account_balance<DB>(
    db: &mut DB,
    from: Address,
    to: Address,
    amount: U256,
) -> Result<(), EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
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

pub fn gas_deltas_and_used<R>(receipts: &[R]) -> Result<(Vec<u64>, u64), EvmAppError>
where
    R: TxReceipt,
{
    let mut previous = 0_u64;
    let mut deltas = Vec::with_capacity(receipts.len());

    for receipt in receipts {
        let cumulative = receipt.cumulative_gas_used();
        let delta = cumulative.checked_sub(previous).ok_or_else(|| {
            EvmAppError::InvalidBlock(format!(
                "receipt cumulative gas must be nondecreasing: previous={previous}, current={cumulative}"
            ))
        })?;
        deltas.push(delta);
        previous = cumulative;
    }

    Ok((deltas, previous))
}

pub fn aggregate_priority_fees(
    txs: &[RecoveredTx],
    gas_deltas: &[u64],
    base_fee_per_gas: u64,
) -> Result<U256, EvmAppError> {
    if txs.len() != gas_deltas.len() {
        return Err(EvmAppError::Execution(format!(
            "priority-fee aggregation requires matching tx/receipt counts, got txs={}, gas_deltas={}",
            txs.len(),
            gas_deltas.len()
        )));
    }

    let mut total = U256::ZERO;
    for (tx, gas_delta) in txs.iter().zip(gas_deltas.iter()) {
        let tip_per_gas = tx.effective_tip_per_gas(base_fee_per_gas).ok_or_else(|| {
            EvmAppError::InvalidBlock("transaction tip under base fee is invalid".into())
        })?;
        let fee = U256::from(*gas_delta)
            .checked_mul(U256::from(tip_per_gas))
            .ok_or_else(|| EvmAppError::Execution("priority-fee multiplication overflow".into()))?;
        total = total
            .checked_add(fee)
            .ok_or_else(|| EvmAppError::Execution("priority-fee accumulation overflow".into()))?;
    }

    Ok(total)
}

pub fn validate_or_recover_fee_recipient(
    evm_config: &WhirlpoolEvmConfig,
    proposer_public_key: [u8; 32],
    carried_fee_recipient: [u8; 20],
) -> Result<Address, EvmAppError> {
    let carried_fee_recipient = Address::from(carried_fee_recipient);
    match evm_config.fee_recipient_for_proposer(proposer_public_key) {
        Some(expected) if expected != carried_fee_recipient => Err(EvmAppError::InvalidBlock(
            format!(
                "proposer fee recipient mismatch for proposer {:?}: expected {expected}, got {carried_fee_recipient}",
                proposer_public_key
            ),
        )),
        Some(expected) => Ok(expected),
        None => Ok(carried_fee_recipient),
    }
}

pub fn extra_data_decode_mode_for_height(
    evm_config: &WhirlpoolEvmConfig,
    block_height: u64,
) -> ExtraDataDecodeMode {
    if block_height >= evm_config.full_dkg_strict_height() {
        ExtraDataDecodeMode::Strict
    } else {
        ExtraDataDecodeMode::Legacy
    }
}

pub fn proposer_public_key_from_raw_eth_section(
    decoded: &CanonicalExtraDataV1,
) -> Result<[u8; 32], EvmAppError> {
    let Some(raw_eth) = decoded.raw_eth.as_ref() else {
        return Err(EvmAppError::InvalidBlock(
            "missing raw_eth section in block extra_data".into(),
        ));
    };
    if raw_eth.len() != 32 {
        return Err(EvmAppError::InvalidBlock(format!(
            "raw_eth proposer key must be 32 bytes, found {}",
            raw_eth.len()
        )));
    }

    let mut proposer_public_key = [0u8; 32];
    proposer_public_key.copy_from_slice(raw_eth);
    Ok(proposer_public_key)
}

pub fn latest_committed_full_dkg<Storage>(
    storage: &Storage,
    start_height: u64,
) -> Result<Option<FullDkgV1>, EvmAppError>
where
    Storage: BlockStorage,
{
    let mut height = start_height;
    loop {
        let maybe_block = storage
            .get_block_by_number(height)
            .map_err(|err| EvmAppError::State(err.to_string()))?;
        if let Some(block) = maybe_block {
            let decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Legacy)
                .map_err(|err| {
                    EvmAppError::InvalidBlock(format!(
                        "failed to decode historical block {height} extra_data: {err}"
                    ))
                })?;
            if let Some(full_dkg) = decoded.full_dkg {
                return Ok(Some(full_dkg));
            }
        }

        if height == 0 {
            break;
        }
        height -= 1;
    }

    Ok(None)
}
