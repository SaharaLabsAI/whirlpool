use alloy_sol_types::{sol, SolCall};
use reth_evm::revm::precompile::PrecompileError;

use crate::community_pool::CommunityPoolPrecompileError;

sol! {
    function communityPoolBalance() external view returns (uint256);
}

pub fn decode_call(data: &[u8]) -> Result<(), PrecompileError> {
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
