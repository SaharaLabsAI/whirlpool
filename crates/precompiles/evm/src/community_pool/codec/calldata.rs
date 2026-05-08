use alloy_primitives::Bytes;
use alloy_sol_types::SolCall;

use crate::community_pool::codec::dispatch::communityPoolBalanceCall;

pub fn community_pool_balance_calldata() -> Bytes {
    Bytes::from(communityPoolBalanceCall {}.abi_encode())
}
