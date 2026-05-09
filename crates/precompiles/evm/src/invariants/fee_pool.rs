use alloy_primitives::{Address, U256};

pub fn withdraw_outcome_preserves_value(
    caller: Address,
    claimable: U256,
    pool_balance_before: Option<U256>,
    caller_balance_before: Option<U256>,
    paid: U256,
    effect: Option<(U256, Address, U256, bool)>,
) -> bool {
    if claimable.is_zero() {
        return paid.is_zero() && effect.is_none();
    }

    let (Some(pool_before), Some(caller_before), Some(effect)) =
        (pool_balance_before, caller_balance_before, effect)
    else {
        return false;
    };
    let (pool_after, effect_caller, caller_after, bump_pool_nonce) = effect;

    paid == claimable
        && effect_caller == caller
        && pool_before.checked_sub(claimable) == Some(pool_after)
        && caller_before.checked_add(claimable) == Some(caller_after)
        && bump_pool_nonce == pool_after.is_zero()
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256};

    use crate::invariants::fee_pool::withdraw_outcome_preserves_value;

    #[test]
    fn withdraw_invariant_rejects_value_mismatch() {
        let caller = Address::repeat_byte(7);

        assert!(withdraw_outcome_preserves_value(
            caller,
            U256::from(5_u64),
            Some(U256::from(10_u64)),
            Some(U256::from(1_u64)),
            U256::from(5_u64),
            Some((U256::from(5_u64), caller, U256::from(6_u64), false)),
        ));
        assert!(!withdraw_outcome_preserves_value(
            caller,
            U256::from(5_u64),
            Some(U256::from(10_u64)),
            Some(U256::from(1_u64)),
            U256::from(4_u64),
            Some((U256::from(5_u64), caller, U256::from(6_u64), false)),
        ));
    }
}
