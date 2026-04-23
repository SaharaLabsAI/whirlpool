use super::*;

impl WhirlpoolEvmConfig {
    pub fn local_proposer_public_key(&self) -> [u8; 32] {
        self.local_proposer_public_key
    }

    pub fn simplex_validators(&self) -> &[ValidatorEntry] {
        &self.simplex_validators
    }

    pub fn simplex_consensus_public_keys(&self) -> Vec<[u8; 32]> {
        self.simplex_validators
            .iter()
            .map(|validator| validator.consensus_pubkey)
            .collect()
    }
}
