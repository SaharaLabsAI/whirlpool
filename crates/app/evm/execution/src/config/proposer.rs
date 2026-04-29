use crate::config::WhirlpoolEvmConfig;

impl WhirlpoolEvmConfig {
    pub fn local_proposer_public_key(&self) -> [u8; 32] {
        self.local_proposer_public_key
    }
}
