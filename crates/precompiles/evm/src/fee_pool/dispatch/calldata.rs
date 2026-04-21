use alloy_primitives::{Address, Bytes};
use alloy_sol_types::SolCall;

use super::{claimableBalanceCall, feePoolBalanceCall, withdrawCall};

pub fn fee_pool_balance_calldata() -> Bytes {
    Bytes::from(feePoolBalanceCall {}.abi_encode())
}

pub fn claimable_balance_calldata(recipient: Address) -> Bytes {
    Bytes::from(claimableBalanceCall { recipient }.abi_encode())
}

pub fn withdraw_calldata() -> Bytes {
    Bytes::from(withdrawCall {}.abi_encode())
}
