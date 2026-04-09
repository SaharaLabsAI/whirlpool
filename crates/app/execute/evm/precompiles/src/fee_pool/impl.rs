use alloy_primitives::{Address, U256};
use reth_evm::precompiles::PrecompileInput;
use revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};

use super::{
    dispatch::{decode_call, FeePoolCall},
    encode_u256_word, gas, revert_result, storage, FeePoolPrecompileError,
    FEE_POOL_PRECOMPILE_ADDRESS,
};

pub(crate) fn execute(mut input: PrecompileInput<'_>) -> PrecompileResult {
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

    let slot = storage::claimable_balance_slot(recipient);
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

    if input.is_static_call() {
        return revert_result(
            gas::WITHDRAW_GAS,
            FeePoolPrecompileError::StaticCallWithdraw,
        );
    }

    let caller = *input.caller();
    let slot = storage::claimable_balance_slot(caller);
    let claimable = input
        .internals_mut()
        .sload(FEE_POOL_PRECOMPILE_ADDRESS, slot)
        .map(|value| value.data)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    if claimable.is_zero() {
        return Ok(PrecompileOutput::new(
            gas::WITHDRAW_GAS,
            encode_u256_word(U256::ZERO),
        ));
    }

    let pool_balance = input
        .internals_mut()
        .load_account(FEE_POOL_PRECOMPILE_ADDRESS)
        .map(|account| account.data.info.balance)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    let next_pool_balance = pool_balance.checked_sub(claimable).ok_or_else(|| {
        PrecompileError::other("withdraw transfer failed: insufficient fee-pool balance")
    })?;

    let caller_balance = input
        .internals_mut()
        .load_account(caller)
        .map(|account| account.data.info.balance)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    let next_caller_balance = caller_balance.checked_add(claimable).ok_or_else(|| {
        PrecompileError::other("withdraw transfer failed: caller balance overflow")
    })?;

    input
        .internals_mut()
        .set_balance(FEE_POOL_PRECOMPILE_ADDRESS, next_pool_balance)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    if next_pool_balance.is_zero() {
        input
            .internals_mut()
            .bump_nonce(FEE_POOL_PRECOMPILE_ADDRESS)
            .map_err(|err| PrecompileError::other(err.to_string()))?;
    }
    input
        .internals_mut()
        .set_balance(caller, next_caller_balance)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    input
        .internals_mut()
        .sstore(FEE_POOL_PRECOMPILE_ADDRESS, slot, U256::ZERO)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    Ok(PrecompileOutput::new(
        gas::WITHDRAW_GAS,
        encode_u256_word(claimable),
    ))
}
