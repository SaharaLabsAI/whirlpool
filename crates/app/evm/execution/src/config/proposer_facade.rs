use crate::config::WhirlpoolEvmConfig;

impl WhirlpoolEvmConfig {
    pub fn with_local_proposer_public_key(mut self, local_proposer_public_key: [u8; 32]) -> Self {
        self.proposer_context = self
            .proposer_context
            .with_local_public_key(local_proposer_public_key);
        self
    }

    pub fn local_proposer_public_key(&self) -> [u8; 32] {
        self.proposer_context.local_public_key()
    }
}
