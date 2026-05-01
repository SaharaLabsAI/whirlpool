use validators_dkg::ValidatorActivationSchedule;

use crate::config::WhirlpoolEvmConfig;

impl WhirlpoolEvmConfig {
    pub fn with_activation_players_for_epoch(mut self, epoch: u64, players: Vec<[u8; 32]>) -> Self {
        self.dkg_transition = self
            .dkg_transition
            .with_activation_players_for_epoch(epoch, players);
        self
    }

    pub fn validator_activation_schedule_for_default_players(
        &self,
        default_players: Vec<[u8; 32]>,
    ) -> ValidatorActivationSchedule {
        self.dkg_transition
            .activation_schedule_for_default_players(default_players)
    }
}
