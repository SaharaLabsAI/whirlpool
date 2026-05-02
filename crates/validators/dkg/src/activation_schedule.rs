use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatorActivationSchedule {
    default_players: Vec<[u8; 32]>,
    overrides_by_epoch: BTreeMap<u64, Vec<[u8; 32]>>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ValidatorActivationError {
    #[error("activation resolver missing player set for epoch {epoch}")]
    MissingPlayers { epoch: u64 },
}

impl ValidatorActivationSchedule {
    pub fn new(default_players: Vec<[u8; 32]>) -> Self {
        Self {
            default_players,
            overrides_by_epoch: BTreeMap::new(),
        }
    }

    pub fn from_parts(
        default_players: Vec<[u8; 32]>,
        overrides_by_epoch: BTreeMap<u64, Vec<[u8; 32]>>,
    ) -> Self {
        Self {
            default_players,
            overrides_by_epoch,
        }
    }
    pub fn resolve_players_for_epoch(
        &self,
        epoch: u64,
    ) -> Result<Vec<[u8; 32]>, ValidatorActivationError> {
        if self.overrides_by_epoch.is_empty() {
            return Ok(self.default_players.clone());
        }

        self.overrides_by_epoch
            .get(&epoch)
            .cloned()
            .ok_or(ValidatorActivationError::MissingPlayers { epoch })
    }
}
