use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{sol, SolCall};

use crate::test_token::TestTokenError;

sol! {
    function mint(address recipient, uint256 amount) external returns (uint256);
    function balanceOf(address account) external view returns (uint256);
}

pub enum TestTokenCall {
    Mint { recipient: Address, amount: U256 },
    BalanceOf { account: Address },
}

pub fn decode_call(data: &[u8]) -> Result<TestTokenCall, TestTokenError> {
    if data.len() < 4 {
        return Err(TestTokenError::CalldataTooShort);
    }

    if data.starts_with(&mintCall::SELECTOR) {
        let call =
            mintCall::abi_decode_validate(data).map_err(|_| TestTokenError::InvalidMintCalldata)?;
        return Ok(TestTokenCall::Mint {
            recipient: call.recipient,
            amount: call.amount,
        });
    }

    if data.starts_with(&balanceOfCall::SELECTOR) {
        let call = balanceOfCall::abi_decode_validate(data)
            .map_err(|_| TestTokenError::InvalidBalanceOfCalldata)?;
        return Ok(TestTokenCall::BalanceOf {
            account: call.account,
        });
    }

    Err(TestTokenError::UnsupportedSelector)
}

pub fn mint_calldata(recipient: Address, amount: U256) -> Bytes {
    Bytes::from(mintCall { recipient, amount }.abi_encode())
}

pub fn balance_of_calldata(account: Address) -> Bytes {
    Bytes::from(balanceOfCall { account }.abi_encode())
}
