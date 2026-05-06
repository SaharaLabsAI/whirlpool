use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{sol, SolCall};
use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};

use crate::RegisteredPrecompile;

mod unlock_accounting;
mod unlock_storage;

pub const COMMUNITY_POOL_ADDRESS: Address = Address::new([
    0x63, 0x6f, 0x6d, 0x6d, 0x75, 0x6e, 0x69, 0x74, 0x79, 0x2d, 0x70, 0x6f, 0x6f, 0x6c, 0x2d, 0x61,
    0x63, 0x63, 0x6f, 0x75,
]);

pub const COMMUNITY_POOL_UNLOCK_EVERY_EPOCHS_SLOT: U256 = U256::ZERO;
pub const COMMUNITY_POOL_UNLOCK_AMOUNT_PER_CYCLE_SLOT: U256 = U256::from_limbs([1, 0, 0, 0]);
pub const COMMUNITY_POOL_LOCKED_REMAINING_SLOT: U256 = U256::from_limbs([2, 0, 0, 0]);
pub const COMMUNITY_POOL_LAST_PROCESSED_EPOCH_SLOT: U256 = U256::from_limbs([3, 0, 0, 0]);

pub use unlock_accounting::{
    apply_post_block_accounting, build_post_block_accounting_effect, CommunityPoolUnlockEffect,
    CommunityPoolUnlockState, PostBlockAccountingEffect, PostBlockAccountingEffectError,
    PostBlockAccountingInputs, PostBlockAccountingOutcome, PostBlockAccountingRuntimeError,
};
pub use unlock_storage::{
    community_pool_last_processed_epoch_slot, community_pool_last_processed_epoch_storage_slot,
    community_pool_locked_remaining_slot, community_pool_locked_remaining_storage_slot,
    community_pool_unlock_amount_per_cycle_slot,
    community_pool_unlock_amount_per_cycle_storage_slot, community_pool_unlock_every_epochs_slot,
    community_pool_unlock_every_epochs_storage_slot, encode_u256_storage_value,
};

sol! {
    function communityPoolBalance() external view returns (uint256);
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CommunityPoolPrecompileError {
    #[error("calldata is too short")]
    CalldataTooShort,
    #[error("unsupported community-pool selector")]
    UnsupportedSelector,
    #[error("invalid community-pool calldata")]
    InvalidCalldata,
    #[error("invalid community-pool return payload")]
    InvalidReturnPayload,
}

pub fn community_pool_balance_calldata() -> Bytes {
    Bytes::from(communityPoolBalanceCall {}.abi_encode())
}

pub fn decode_community_pool_balance_output(
    payload: &Bytes,
) -> Result<U256, CommunityPoolPrecompileError> {
    if payload.len() != 32 {
        return Err(CommunityPoolPrecompileError::InvalidReturnPayload);
    }

    let mut word = [0u8; 32];
    word.copy_from_slice(payload.as_ref());
    Ok(U256::from_be_bytes(word))
}

pub fn register() -> RegisteredPrecompile {
    RegisteredPrecompile::new_stateful(
        "whirlpool_community_pool_balance",
        COMMUNITY_POOL_ADDRESS,
        execute,
    )
}

fn execute(mut input: PrecompileInput<'_>) -> PrecompileResult {
    let gas_limit = input.gas();
    if gas_limit < gas::COMMUNITY_POOL_BALANCE_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    decode_call(input.data())?;

    let balance = input
        .internals_mut()
        .load_account(COMMUNITY_POOL_ADDRESS)
        .map(|account| account.data.info.balance)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    Ok(PrecompileOutput::new(
        gas::COMMUNITY_POOL_BALANCE_GAS,
        encode_u256_word(balance),
    ))
}

fn decode_call(data: &[u8]) -> Result<(), PrecompileError> {
    if data.len() < 4 {
        return Err(PrecompileError::other(
            CommunityPoolPrecompileError::CalldataTooShort.to_string(),
        ));
    }

    if !data.starts_with(&communityPoolBalanceCall::SELECTOR) {
        return Err(PrecompileError::other(
            CommunityPoolPrecompileError::UnsupportedSelector.to_string(),
        ));
    }

    communityPoolBalanceCall::abi_decode_validate(data)
        .map(|_| ())
        .map_err(|_| {
            PrecompileError::other(CommunityPoolPrecompileError::InvalidCalldata.to_string())
        })
}

fn encode_u256_word(value: U256) -> Bytes {
    Bytes::copy_from_slice(&value.to_be_bytes::<32>())
}

pub mod gas {
    pub const COMMUNITY_POOL_BALANCE_GAS: u64 = 750;
}

#[cfg(test)]
mod tests;
