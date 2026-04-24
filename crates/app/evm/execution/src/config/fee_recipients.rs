use alloy_primitives::Address;

use crate::config::{WhirlpoolEvmConfig, DEFAULT_PROPOSER_FEE_RECIPIENT};

impl WhirlpoolEvmConfig {
    pub fn fee_recipient(&self) -> Address {
        self.fee_recipient_for_proposer(self.local_proposer_public_key)
            .unwrap_or(DEFAULT_PROPOSER_FEE_RECIPIENT)
    }

    pub fn fee_recipient_for_proposer(&self, proposer_public_key: [u8; 32]) -> Option<Address> {
        self.validator_fee_recipients
            .get(&proposer_public_key)
            .copied()
    }
}
