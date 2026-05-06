use alloy_primitives::{Address, U256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithdrawInput {
    pub caller: Address,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithdrawState {
    pub claimable: U256,
    pub balances: Option<WithdrawBalances>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithdrawBalances {
    pub pool: U256,
    pub caller: U256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithdrawEffect {
    pub pool_balance: U256,
    pub caller: Address,
    pub caller_balance: U256,
    pub bump_pool_nonce: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithdrawOutcome {
    pub paid: U256,
    pub effect: Option<WithdrawEffect>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WithdrawTransitionError {
    #[error("withdraw transfer failed: missing balance snapshot")]
    MissingBalanceSnapshot,
    #[error("withdraw transfer failed: insufficient fee-pool balance")]
    InsufficientFeePoolBalance,
    #[error("withdraw transfer failed: caller balance overflow")]
    CallerBalanceOverflow,
}

pub fn plan_withdraw(
    input: WithdrawInput,
    state: WithdrawState,
) -> Result<WithdrawOutcome, WithdrawTransitionError> {
    if state.claimable.is_zero() {
        return Ok(WithdrawOutcome {
            paid: U256::ZERO,
            effect: None,
        });
    }

    let balances = state
        .balances
        .ok_or(WithdrawTransitionError::MissingBalanceSnapshot)?;
    let pool_balance = balances
        .pool
        .checked_sub(state.claimable)
        .ok_or(WithdrawTransitionError::InsufficientFeePoolBalance)?;
    let caller_balance = balances
        .caller
        .checked_add(state.claimable)
        .ok_or(WithdrawTransitionError::CallerBalanceOverflow)?;

    Ok(WithdrawOutcome {
        paid: state.claimable,
        effect: Some(WithdrawEffect {
            pool_balance,
            caller: input.caller,
            caller_balance,
            bump_pool_nonce: pool_balance.is_zero(),
        }),
    })
}

#[cfg(test)]
mod tests;
