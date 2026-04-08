use alloy_primitives::{Address, U256};
use reth_evm::precompiles::PrecompileInput;
use revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};

use super::{
    dispatch::{decode_call, TestTokenCall},
    encode_u256_word, gas, revert_result, TestTokenError,
};

pub(crate) fn execute(input: PrecompileInput<'_>) -> PrecompileResult {
    let gas_limit = input.gas();
    let dispatch = match decode_call(input.data()) {
        Ok(dispatch) => dispatch,
        Err(error) => return revert_result(gas::BALANCE_OF_GAS, error),
    };

    match dispatch {
        TestTokenCall::Mint { recipient, amount } => mint(input, gas_limit, recipient, amount),
        TestTokenCall::BalanceOf { account } => balance_of(input, gas_limit, account),
    }
}

fn mint(
    mut input: PrecompileInput<'_>,
    gas_limit: u64,
    recipient: Address,
    amount: U256,
) -> PrecompileResult {
    if gas_limit < gas::MINT_GAS {
        return Err(PrecompileError::OutOfGas);
    }
    if input.is_static_call() {
        return revert_result(gas::MINT_GAS, TestTokenError::StaticCall);
    }
    if amount.is_zero() {
        return revert_result(gas::MINT_GAS, TestTokenError::ZeroAmount);
    }

    let current_balance = load_account_balance(&mut input, recipient)?;
    let next_balance = current_balance
        .checked_add(amount)
        .ok_or(TestTokenError::ArithmeticOverflow)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    input
        .internals_mut()
        .balance_incr(recipient, amount)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    Ok(PrecompileOutput::new(
        gas::MINT_GAS,
        encode_u256_word(next_balance),
    ))
}

fn balance_of(
    mut input: PrecompileInput<'_>,
    gas_limit: u64,
    account: Address,
) -> PrecompileResult {
    if gas_limit < gas::BALANCE_OF_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    let balance = load_account_balance(&mut input, account)?;
    Ok(PrecompileOutput::new(
        gas::BALANCE_OF_GAS,
        encode_u256_word(balance),
    ))
}

fn load_account_balance(
    input: &mut PrecompileInput<'_>,
    address: Address,
) -> Result<U256, PrecompileError> {
    input
        .internals_mut()
        .load_account(address)
        .map(|account| account.data.info.balance)
        .map_err(|err| PrecompileError::other(err.to_string()))
}
