use std::fmt::Display;

use state::StateDb;

use crate::community_pool::PostBlockAccountingRuntimeError;
use crate::fee_pool::{claimable_balance_slot, ClaimCredit, FEE_POOL_PRECOMPILE_ADDRESS};

pub fn credit_fee_pool_claim<DB>(
    db: &mut DB,
    claim: &ClaimCredit,
) -> Result<(), PostBlockAccountingRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    if claim.amount.is_zero() {
        return Ok(());
    }

    let slot = claimable_balance_slot(claim.recipient);
    let current = db
        .get_storage(FEE_POOL_PRECOMPILE_ADDRESS, slot)
        .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))?;
    let next = current.checked_add(claim.amount).ok_or_else(|| {
        PostBlockAccountingRuntimeError::Execution("fee-pool claim ledger overflow".into())
    })?;

    db.insert_storage(FEE_POOL_PRECOMPILE_ADDRESS, slot, next)
        .map_err(|err| PostBlockAccountingRuntimeError::StateAccess(err.to_string()))
}
