use alloy_primitives::{address, Address, Bytes, U256};
use revm::precompile::PrecompileResult;

use crate::{RegisteredPrecompile, WhirlpoolStatefulPrecompile};

mod dispatch;
pub mod gas;
mod r#impl;

pub const TEST_TOKEN_PRECOMPILE_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000100");

pub use dispatch::{balance_of_calldata, mint_calldata};

#[derive(Debug, thiserror::Error)]
pub enum TestTokenError {
    #[error("calldata is too short")]
    CalldataTooShort,
    #[error("unsupported test-token selector")]
    UnsupportedSelector,
    #[error("invalid mint calldata")]
    InvalidMintCalldata,
    #[error("invalid balanceOf calldata")]
    InvalidBalanceOfCalldata,
    #[error("mint amount must be non-zero")]
    ZeroAmount,
    #[error("mint cannot run in a static context")]
    StaticCall,
    #[error("token arithmetic overflow")]
    ArithmeticOverflow,
}

/// Validation/example Whirlpool precompile that uses the framework-owned
/// direct-call-only registration path. Indirect calls are rejected by the
/// shared crate-level guard before this module's business logic runs.
pub struct TestTokenPrecompile;

impl WhirlpoolStatefulPrecompile for TestTokenPrecompile {
    fn register() -> RegisteredPrecompile {
        RegisteredPrecompile::new_stateful(
            "whirlpool_test_token",
            TEST_TOKEN_PRECOMPILE_ADDRESS,
            r#impl::execute,
        )
    }
}

fn encode_u256_word(value: U256) -> Bytes {
    Bytes::copy_from_slice(&value.to_be_bytes::<32>())
}

fn encode_revert_reason(reason: &str) -> Bytes {
    let reason_bytes = reason.as_bytes();
    let padded_len = ((reason_bytes.len() + 31) / 32) * 32;
    let mut payload = Vec::with_capacity(4 + 32 * 3 + padded_len);
    payload.extend_from_slice(&[0x08, 0xc3, 0x79, 0xa0]);
    payload.extend_from_slice(&U256::from(32_u64).to_be_bytes::<32>());
    payload.extend_from_slice(&U256::from(reason_bytes.len()).to_be_bytes::<32>());
    payload.extend_from_slice(reason_bytes);
    payload.resize(4 + 32 * 2 + padded_len, 0);
    Bytes::from(payload)
}

fn revert_result(gas_used: u64, error: TestTokenError) -> PrecompileResult {
    Ok(revm::precompile::PrecompileOutput::new_reverted(
        gas_used,
        encode_revert_reason(&error.to_string()),
    ))
}
