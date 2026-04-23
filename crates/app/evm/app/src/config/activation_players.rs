use super::*;

impl WhirlpoolEvmConfig {
    pub fn with_activation_players_for_epoch(mut self, epoch: u64, players: Vec<[u8; 32]>) -> Self {
        self.activation_players_by_epoch.insert(epoch, players);
        self
    }

    pub fn activation_players_for_epoch(&self, epoch: u64) -> Option<Vec<[u8; 32]>> {
        if self.activation_players_by_epoch.is_empty() {
            return Some(self.simplex_consensus_public_keys());
        }
        self.activation_players_by_epoch.get(&epoch).cloned()
    }
}
