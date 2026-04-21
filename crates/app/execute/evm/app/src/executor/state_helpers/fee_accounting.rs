use alloy_consensus::Transaction;
use alloy_primitives::{Address, U256};
use evm_precompiles::{
    claimable_balance_slot, COMMUNITY_POOL_ADDRESS, FEE_POOL_PRECOMPILE_ADDRESS,
};

use crate::{error::EvmAppError, traits::StateProvider};

use super::super::RecoveredTx;
use super::account_balances::credit_account_balance;

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
