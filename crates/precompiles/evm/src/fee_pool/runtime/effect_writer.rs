use alloy_primitives::U256;
use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::PrecompileError;

use crate::fee_pool::{transition::withdraw::WithdrawEffect, FEE_POOL_PRECOMPILE_ADDRESS};

pub fn apply_withdraw_effect(
    input: &mut PrecompileInput<'_>,
    slot: U256,
    effect: WithdrawEffect,
) -> Result<(), PrecompileError> {
    input
        .internals_mut()
        .set_balance(FEE_POOL_PRECOMPILE_ADDRESS, effect.pool_balance)
        .map_err(|err| PrecompileError::other(err.to_string()))?;
    if effect.bump_pool_nonce {
        input
            .internals_mut()
            .bump_nonce(FEE_POOL_PRECOMPILE_ADDRESS)
            .map_err(|err| PrecompileError::other(err.to_string()))?;
    }
    input
        .internals_mut()
        .set_balance(effect.caller, effect.caller_balance)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    input
        .internals_mut()
        .sstore(FEE_POOL_PRECOMPILE_ADDRESS, slot, U256::ZERO)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    Ok(())
}
