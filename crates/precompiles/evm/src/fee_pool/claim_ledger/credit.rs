use alloy_primitives::{Address, U256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimCredit {
    pub recipient: Address,
    pub amount: U256,
}
