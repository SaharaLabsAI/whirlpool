use alloy_primitives::Address;
use alloy_sol_types::{sol, SolCall};

use crate::fee_pool::FeePoolPrecompileError;

sol! {
    function feePoolBalance() external view returns (uint256);
    function claimableBalance(address recipient) external view returns (uint256);
    function withdraw() external returns (uint256);
}

pub enum FeePoolCall {
    FeePoolBalance,
    ClaimableBalance { recipient: Address },
    Withdraw,
}

pub fn decode_call(data: &[u8]) -> Result<FeePoolCall, FeePoolPrecompileError> {
    if data.len() < 4 {
        return Err(FeePoolPrecompileError::CalldataTooShort);
    }

    if data.starts_with(&feePoolBalanceCall::SELECTOR) {
        feePoolBalanceCall::abi_decode_validate(data)
            .map_err(|_| FeePoolPrecompileError::InvalidFeePoolBalanceCalldata)?;
        return Ok(FeePoolCall::FeePoolBalance);
    }

    if data.starts_with(&claimableBalanceCall::SELECTOR) {
        let call = claimableBalanceCall::abi_decode_validate(data)
            .map_err(|_| FeePoolPrecompileError::InvalidClaimableBalanceCalldata)?;
        return Ok(FeePoolCall::ClaimableBalance {
            recipient: call.recipient,
        });
    }

    if data.starts_with(&withdrawCall::SELECTOR) {
        withdrawCall::abi_decode_validate(data)
            .map_err(|_| FeePoolPrecompileError::InvalidWithdrawCalldata)?;
        return Ok(FeePoolCall::Withdraw);
    }

    Err(FeePoolPrecompileError::UnsupportedSelector)
}
