use validators_dkg::ValidatorActivationSchedule;

use crate::config::WhirlpoolEvmConfig;

impl WhirlpoolEvmConfig {
    pub fn with_activation_players_for_epoch(mut self, epoch: u64, players: Vec<[u8; 32]>) -> Self {
        self.activation_players_by_epoch.insert(epoch, players);
        self
    }

    pub fn validator_activation_schedule(&self) -> ValidatorActivationSchedule {
        ValidatorActivationSchedule::from_parts(
            self.simplex_consensus_public_keys(),
            self.activation_players_by_epoch.clone(),
        )
    }

    pub fn activation_players_for_epoch(&self, epoch: u64) -> Option<Vec<[u8; 32]>> {
        self.validator_activation_schedule()
            .resolve_players_for_epoch(epoch)
            .ok()
    }
}
