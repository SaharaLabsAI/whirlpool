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
