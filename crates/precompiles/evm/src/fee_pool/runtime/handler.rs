use alloy_primitives::Address;
use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};

use crate::fee_pool::{
    claim_ledger::claimable_balance_slot,
    codec::{decode_call, FeePoolCall},
    encode_u256_word, gas, revert_result,
    transition::withdraw::{plan_withdraw, WithdrawInput, WithdrawState},
    FeePoolPrecompileError, FEE_POOL_PRECOMPILE_ADDRESS,
};

use crate::fee_pool::runtime::{
    effect_writer::apply_withdraw_effect, state::load_withdraw_balances,
};

pub fn execute(mut input: PrecompileInput<'_>) -> PrecompileResult {
    let gas_limit = input.gas();
    let dispatch = match decode_call(input.data()) {
        Ok(call) => call,
        Err(error) => return revert_result(gas::CLAIMABLE_BALANCE_GAS, error),
    };

    match dispatch {
        FeePoolCall::FeePoolBalance => fee_pool_balance(&mut input, gas_limit),
        FeePoolCall::ClaimableBalance { recipient } => {
            claimable_balance(&mut input, gas_limit, recipient)
        }
        FeePoolCall::Withdraw => withdraw(input, gas_limit),
    }
}

fn fee_pool_balance(input: &mut PrecompileInput<'_>, gas_limit: u64) -> PrecompileResult {
    if gas_limit < gas::FEE_POOL_BALANCE_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    let balance = input
        .internals_mut()
        .load_account(FEE_POOL_PRECOMPILE_ADDRESS)
        .map(|account| account.data.info.balance)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    Ok(PrecompileOutput::new(
        gas::FEE_POOL_BALANCE_GAS,
        encode_u256_word(balance),
    ))
}

fn claimable_balance(
    input: &mut PrecompileInput<'_>,
    gas_limit: u64,
    recipient: Address,
) -> PrecompileResult {
    if gas_limit < gas::CLAIMABLE_BALANCE_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    let slot = claimable_balance_slot(recipient);
    let claimable = input
        .internals_mut()
        .sload(FEE_POOL_PRECOMPILE_ADDRESS, slot)
        .map(|value| value.data)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    Ok(PrecompileOutput::new(
        gas::CLAIMABLE_BALANCE_GAS,
        encode_u256_word(claimable),
    ))
}

fn withdraw(mut input: PrecompileInput<'_>, gas_limit: u64) -> PrecompileResult {
    if gas_limit < gas::WITHDRAW_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    if !crate::invariants::call_boundary::write_call_is_not_static(input.is_static_call()) {
        return revert_result(
            gas::WITHDRAW_GAS,
            FeePoolPrecompileError::StaticCallWithdraw,
        );
    }

    let caller = *input.caller();
    let slot = claimable_balance_slot(caller);
    let claimable = input
        .internals_mut()
        .sload(FEE_POOL_PRECOMPILE_ADDRESS, slot)
        .map(|value| value.data)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    let balances = Some(load_withdraw_balances(&mut input, caller)?);
    let withdraw_input = WithdrawInput { caller };
    let withdraw_state = WithdrawState {
        claimable,
        balances,
    };
    let outcome = plan_withdraw(withdraw_input, withdraw_state)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    if !crate::invariants::fee_pool::withdraw_outcome_preserves_value(
        caller,
        claimable,
        balances.map(|snapshot| snapshot.pool),
        balances.map(|snapshot| snapshot.caller),
        outcome.paid,
        outcome.effect.map(|effect| {
            (
                effect.pool_balance,
                effect.caller,
                effect.caller_balance,
                effect.bump_pool_nonce,
            )
        }),
    ) {
        return Err(PrecompileError::other(
            "fee-pool withdraw invariant violation",
        ));
    }

    if let Some(effect) = outcome.effect {
        apply_withdraw_effect(&mut input, slot, effect)?;
    }

    Ok(PrecompileOutput::new(
        gas::WITHDRAW_GAS,
        encode_u256_word(outcome.paid),
    ))
}
