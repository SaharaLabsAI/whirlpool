use std::collections::BTreeMap;

use crate::epoch::EpochActivationTargets;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatorActivationSchedule {
    default_players: Vec<[u8; 32]>,
    overrides_by_epoch: BTreeMap<u64, Vec<[u8; 32]>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundaryValidatorActivation {
    pub targets: EpochActivationTargets,
    pub full_dkg_players: Vec<[u8; 32]>,
    pub reshare_players: Vec<[u8; 32]>,
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

    pub fn with_epoch_players(mut self, epoch: u64, players: Vec<[u8; 32]>) -> Self {
        self.overrides_by_epoch.insert(epoch, players);
        self
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

    pub fn resolve_boundary_activation(
        &self,
        targets: EpochActivationTargets,
    ) -> Result<BoundaryValidatorActivation, ValidatorActivationError> {
        Ok(BoundaryValidatorActivation {
            targets,
            full_dkg_players: self.resolve_players_for_epoch(targets.full_dkg_epoch)?,
            reshare_players: self.resolve_players_for_epoch(targets.reshare_target_epoch)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_schedule_resolves_any_epoch_to_simplex_players() {
        let schedule = ValidatorActivationSchedule::new(vec![[0x11; 32], [0x22; 32]]);

        assert_eq!(
            schedule.resolve_players_for_epoch(42).expect("players"),
            vec![[0x11; 32], [0x22; 32]]
        );
    }

    #[test]
    fn override_schedule_is_strict_and_fail_closed() {
        let schedule = ValidatorActivationSchedule::new(vec![[0x11; 32]])
            .with_epoch_players(2, vec![[0x21; 32]])
            .with_epoch_players(3, vec![[0x31; 32], [0x32; 32]]);

        assert_eq!(
            schedule.resolve_players_for_epoch(2).expect("epoch 2"),
            vec![[0x21; 32]]
        );
        assert_eq!(
            schedule.resolve_players_for_epoch(4),
            Err(ValidatorActivationError::MissingPlayers { epoch: 4 })
        );
    }

    #[test]
    fn boundary_activation_consumes_epoch_targets() {
        let targets = EpochActivationTargets::from_post_advance_epoch(7).expect("targets");
        let schedule = ValidatorActivationSchedule::new(vec![[0x11; 32]])
            .with_epoch_players(targets.full_dkg_epoch, vec![[0x81; 32]])
            .with_epoch_players(targets.reshare_target_epoch, vec![[0x91; 32]]);

        let activation = schedule
            .resolve_boundary_activation(targets)
            .expect("boundary activation");

        assert_eq!(activation.targets, targets);
        assert_eq!(activation.full_dkg_players, vec![[0x81; 32]]);
        assert_eq!(activation.reshare_players, vec![[0x91; 32]]);
    }
}
