use alloy_consensus::Transaction;
use alloy_primitives::U256;

use crate::error::EvmAppError;

use super::super::RecoveredTx;

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
