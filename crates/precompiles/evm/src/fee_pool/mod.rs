use alloy_primitives::{address, Address, Bytes, U256};
use reth_evm::revm::precompile::PrecompileResult;

use crate::RegisteredPrecompile;

#[doc(hidden)]
pub mod claim_ledger;
mod codec;
pub mod gas;
mod runtime;
pub mod storage;
mod transition;

pub use claim_ledger::ClaimCredit;
pub use codec::{
    claimable_balance_calldata, decode_claimable_balance_output, decode_fee_pool_balance_output,
    decode_withdraw_output, fee_pool_balance_calldata, withdraw_calldata,
};
pub use storage::claimable_balance_slot;

pub const FEE_POOL_PRECOMPILE_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000102");

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FeePoolPrecompileError {
    #[error("calldata is too short")]
    CalldataTooShort,
    #[error("unsupported fee-pool selector")]
    UnsupportedSelector,
    #[error("invalid feePoolBalance calldata")]
    InvalidFeePoolBalanceCalldata,
    #[error("invalid claimableBalance calldata")]
    InvalidClaimableBalanceCalldata,
    #[error("invalid withdraw calldata")]
    InvalidWithdrawCalldata,
    #[error("withdraw cannot run in a static context")]
    StaticCallWithdraw,
    #[error("invalid fee-pool return payload")]
    InvalidReturnPayload,
}

pub fn register() -> RegisteredPrecompile {
    RegisteredPrecompile::new_stateful(
        "whirlpool_fee_pool",
        FEE_POOL_PRECOMPILE_ADDRESS,
        runtime::handler::execute,
    )
}

fn encode_u256_word(value: U256) -> Bytes {
    Bytes::copy_from_slice(&value.to_be_bytes::<32>())
}

fn encode_revert_reason(reason: &str) -> Bytes {
    let reason_bytes = reason.as_bytes();
    let padded_len = reason_bytes.len().div_ceil(32) * 32;
    let mut payload = Vec::with_capacity(4 + 32 * 3 + padded_len);
    payload.extend_from_slice(&[0x08, 0xc3, 0x79, 0xa0]);
    payload.extend_from_slice(&U256::from(32_u64).to_be_bytes::<32>());
    payload.extend_from_slice(&U256::from(reason_bytes.len()).to_be_bytes::<32>());
    payload.extend_from_slice(reason_bytes);
    payload.resize(4 + 32 * 2 + padded_len, 0);
    Bytes::from(payload)
}

fn revert_result(gas_used: u64, error: FeePoolPrecompileError) -> PrecompileResult {
    Ok(reth_evm::revm::precompile::PrecompileOutput::new_reverted(
        gas_used,
        encode_revert_reason(&error.to_string()),
    ))
}

#[cfg(test)]
mod tests;
