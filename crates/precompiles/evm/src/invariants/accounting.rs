use alloy_primitives::U256;

use crate::community_pool::{
    CommunityPoolUnlockState, PostBlockAccountingEffect, PostBlockAccountingInputs,
};
use crate::invariants::community_pool::unlock_effect_is_consistent;

pub fn checked_balance_credit(balance: U256, amount: U256) -> Option<U256> {
    balance.checked_add(amount)
}

pub fn post_block_accounting_effect_matches_inputs(
    inputs: &PostBlockAccountingInputs,
    current_epoch: u64,
    unlock_state: &CommunityPoolUnlockState,
    effect: &PostBlockAccountingEffect,
) -> bool {
    let expected_burned = U256::from(inputs.gas_used) * U256::from(inputs.base_fee_per_gas);
    if effect.burned_fees != expected_burned {
        return false;
    }

    match (&effect.priority_fee_claim, inputs.priority_fees.is_zero()) {
        (None, true) => {}
        (Some(claim), false)
            if claim.recipient == inputs.claim_recipient
                && claim.amount == inputs.priority_fees => {}
        _ => return false,
    }

    match &effect.community_pool_unlock {
        Some(unlock) => unlock_effect_is_consistent(
            unlock_state,
            current_epoch,
            inputs.simplex_validators.len(),
            unlock,
        ),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;

    use crate::invariants::accounting::checked_balance_credit;

    #[test]
    fn checked_balance_credit_rejects_overflow() {
        assert_eq!(checked_balance_credit(U256::MAX, U256::from(1_u64)), None);
        assert_eq!(
            checked_balance_credit(U256::from(4_u64), U256::from(5_u64)),
            Some(U256::from(9_u64))
        );
    }
}
