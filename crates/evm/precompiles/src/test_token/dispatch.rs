use alloy_primitives::{Address, Bytes, U256};

use crate::test_token::TestTokenError;

pub const MINT_SELECTOR: [u8; 4] = [0x40, 0xc1, 0x0f, 0x19];
pub const BALANCE_OF_SELECTOR: [u8; 4] = [0x70, 0xa0, 0x82, 0x31];

pub enum TestTokenCall {
    Mint { recipient: Address, amount: U256 },
    BalanceOf { account: Address },
}

pub fn decode_call(data: &[u8]) -> Result<TestTokenCall, TestTokenError> {
    if data.len() < 4 {
        return Err(TestTokenError::CalldataTooShort);
    }

    let selector: [u8; 4] = data[..4].try_into().expect("selector slice");
    match selector {
        MINT_SELECTOR => {
            if data.len() != 4 + 32 + 32 {
                return Err(TestTokenError::InvalidMintCalldata);
            }
            Ok(TestTokenCall::Mint {
                recipient: decode_address(&data[4..36]).ok_or(TestTokenError::InvalidMintCalldata)?,
                amount: decode_u256(&data[36..68]).ok_or(TestTokenError::InvalidMintCalldata)?,
            })
        }
        BALANCE_OF_SELECTOR => {
            if data.len() != 4 + 32 {
                return Err(TestTokenError::InvalidBalanceOfCalldata);
            }
            Ok(TestTokenCall::BalanceOf {
                account: decode_address(&data[4..36])
                    .ok_or(TestTokenError::InvalidBalanceOfCalldata)?,
            })
        }
        _ => Err(TestTokenError::UnsupportedSelector),
    }
}

pub fn mint_calldata(recipient: Address, amount: U256) -> Bytes {
    let mut data = Vec::with_capacity(4 + 32 + 32);
    data.extend_from_slice(&MINT_SELECTOR);
    data.extend_from_slice(&encode_address_word(recipient));
    data.extend_from_slice(&amount.to_be_bytes::<32>());
    Bytes::from(data)
}

pub fn balance_of_calldata(account: Address) -> Bytes {
    let mut data = Vec::with_capacity(4 + 32);
    data.extend_from_slice(&BALANCE_OF_SELECTOR);
    data.extend_from_slice(&encode_address_word(account));
    Bytes::from(data)
}

fn encode_address_word(address: Address) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[12..].copy_from_slice(address.as_slice());
    word
}

fn decode_address(word: &[u8]) -> Option<Address> {
    (word.len() == 32).then(|| Address::from_slice(&word[12..]))
}

fn decode_u256(word: &[u8]) -> Option<U256> {
    (word.len() == 32).then(|| {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(word);
        U256::from_be_bytes(bytes)
    })
}
