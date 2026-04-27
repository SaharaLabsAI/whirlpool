#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochActivationTargets {
    pub boundary_epoch_e: u64,
    pub full_dkg_epoch: u64,
    pub reshare_target_epoch: u64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EpochActivationTargetError {
    #[error("full_dkg epoch overflow at boundary")]
    FullDkgEpochOverflow,
    #[error("reshare target epoch overflow at boundary")]
    ReshareTargetEpochOverflow,
}

impl EpochActivationTargets {
    pub fn from_post_advance_epoch(
        post_advance_epoch: u64,
    ) -> Result<Self, EpochActivationTargetError> {
        let full_dkg_epoch = post_advance_epoch
            .checked_add(1)
            .ok_or(EpochActivationTargetError::FullDkgEpochOverflow)?;
        let reshare_target_epoch = post_advance_epoch
            .checked_add(2)
            .ok_or(EpochActivationTargetError::ReshareTargetEpochOverflow)?;

        Ok(Self {
            boundary_epoch_e: post_advance_epoch,
            full_dkg_epoch,
            reshare_target_epoch,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_targets_are_forward_looking_from_post_advance_epoch() {
        let targets = EpochActivationTargets::from_post_advance_epoch(7).expect("targets");

        assert_eq!(targets.boundary_epoch_e, 7);
        assert_eq!(targets.full_dkg_epoch, 8);
        assert_eq!(targets.reshare_target_epoch, 9);
    }

    #[test]
    fn activation_targets_fail_closed_on_overflow() {
        assert_eq!(
            EpochActivationTargets::from_post_advance_epoch(u64::MAX),
            Err(EpochActivationTargetError::FullDkgEpochOverflow)
        );
        assert_eq!(
            EpochActivationTargets::from_post_advance_epoch(u64::MAX - 1),
            Err(EpochActivationTargetError::ReshareTargetEpochOverflow)
        );
    }
}
