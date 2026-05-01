use std::collections::BTreeMap;

use validators_dkg::ValidatorActivationSchedule;

#[derive(Debug, Clone, Default)]
pub struct DkgActivationOverrides {
    players_by_epoch: BTreeMap<u64, Vec<[u8; 32]>>,
}

impl DkgActivationOverrides {
    pub fn with_players_for_epoch(mut self, epoch: u64, players: Vec<[u8; 32]>) -> Self {
        self.players_by_epoch.insert(epoch, players);
        self
    }

    pub fn schedule_for_default_players(
        &self,
        default_players: Vec<[u8; 32]>,
    ) -> ValidatorActivationSchedule {
        ValidatorActivationSchedule::from_parts(default_players, self.players_by_epoch.clone())
    }
}
