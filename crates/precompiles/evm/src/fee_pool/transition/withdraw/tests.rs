use alloy_primitives::{Address, U256};

use crate::fee_pool::transition::withdraw::{
    plan_withdraw, WithdrawBalances, WithdrawInput, WithdrawState, WithdrawTransitionError,
};

#[test]
fn zero_claim_is_noop_without_balance_snapshot() {
    let outcome = plan_withdraw(
        WithdrawInput {
            caller: Address::repeat_byte(0x11),
        },
        WithdrawState {
            claimable: U256::ZERO,
            balances: None,
        },
    )
    .expect("zero claim should plan");

    assert_eq!(outcome.paid, U256::ZERO);
    assert_eq!(outcome.effect, None);
}

#[test]
fn nonzero_claim_plans_transfer_and_claim_clear() {
    let caller = Address::repeat_byte(0x22);
    let outcome = plan_withdraw(
        WithdrawInput { caller },
        WithdrawState {
            claimable: U256::from(5_u64),
            balances: Some(WithdrawBalances {
                pool: U256::from(7_u64),
                caller: U256::from(11_u64),
            }),
        },
    )
    .expect("withdraw should plan");
    let effect = outcome.effect.expect("nonzero claim has effect");

    assert_eq!(outcome.paid, U256::from(5_u64));
    assert_eq!(effect.pool_balance, U256::from(2_u64));
    assert_eq!(effect.caller, caller);
    assert_eq!(effect.caller_balance, U256::from(16_u64));
    assert!(!effect.bump_pool_nonce);
}

#[test]
fn insufficient_pool_balance_is_rejected_by_planner() {
    let err = plan_withdraw(
        WithdrawInput {
            caller: Address::repeat_byte(0x33),
        },
        WithdrawState {
            claimable: U256::from(8_u64),
            balances: Some(WithdrawBalances {
                pool: U256::from(7_u64),
                caller: U256::ZERO,
            }),
        },
    )
    .expect_err("insufficient pool should fail");

    assert_eq!(err, WithdrawTransitionError::InsufficientFeePoolBalance);
}

#[test]
fn withdraw_invariant_rejects_value_mismatch() {
    let caller = Address::repeat_byte(0x44);

    assert!(
        !crate::invariants::fee_pool::withdraw_outcome_preserves_value(
            caller,
            U256::from(5_u64),
            Some(U256::from(7_u64)),
            Some(U256::from(11_u64)),
            U256::from(4_u64),
            Some((U256::from(2_u64), caller, U256::from(16_u64), false)),
        )
    );
}
