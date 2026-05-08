use alloy_primitives::Bytes;
use alloy_sol_types::SolCall;

use crate::validators::codec::dispatch::validatorsCall;

pub fn validators_calldata() -> Bytes {
    Bytes::from(validatorsCall {}.abi_encode())
}
