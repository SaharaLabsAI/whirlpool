use validators_reader::ValidatorEntry;

use crate::config::WhirlpoolEvmConfig;

impl WhirlpoolEvmConfig {
    pub fn local_proposer_public_key(&self) -> [u8; 32] {
        self.local_proposer_public_key
    }

    pub fn validator_registry_entries(&self) -> &[ValidatorEntry] {
        &self.validator_registry_entries
    }

    pub fn validator_consensus_public_keys(&self) -> Vec<[u8; 32]> {
        self.validator_registry_entries
            .iter()
            .map(|validator| validator.consensus_pubkey)
            .collect()
    }
}
