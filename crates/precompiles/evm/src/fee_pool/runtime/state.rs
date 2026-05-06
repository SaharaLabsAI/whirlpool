use alloy_primitives::Address;
use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::PrecompileError;

use crate::fee_pool::{transition::withdraw::WithdrawBalances, FEE_POOL_PRECOMPILE_ADDRESS};

pub fn load_withdraw_balances(
    input: &mut PrecompileInput<'_>,
    caller: Address,
) -> Result<WithdrawBalances, PrecompileError> {
    let pool = input
        .internals_mut()
        .load_account(FEE_POOL_PRECOMPILE_ADDRESS)
        .map(|account| account.data.info.balance)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    let caller = input
        .internals_mut()
        .load_account(caller)
        .map(|account| account.data.info.balance)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    Ok(WithdrawBalances { pool, caller })
}
