use crate::error::EvmAppError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundaryEpochContext {
    pub boundary_epoch_e: u64,
    pub full_dkg_epoch: u64,
    pub reshare_target_epoch: u64,
}

impl BoundaryEpochContext {
    pub fn from_post_advance_epoch(post_advance_epoch: u64) -> Result<Self, EvmAppError> {
        let full_dkg_epoch = post_advance_epoch.checked_add(1).ok_or_else(|| {
            EvmAppError::InvalidBlock("full_dkg epoch overflow at boundary".into())
        })?;
        let reshare_target_epoch = post_advance_epoch.checked_add(2).ok_or_else(|| {
            EvmAppError::InvalidBlock("reshare target epoch overflow at boundary".into())
        })?;

        Ok(Self {
            boundary_epoch_e: post_advance_epoch,
            full_dkg_epoch,
            reshare_target_epoch,
        })
    }
}
