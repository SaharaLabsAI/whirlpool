use crate::config::WhirlpoolEvmConfig;
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

#[derive(Clone, Copy, Debug)]
pub struct ActivationSourceResolver<'a> {
    evm_config: &'a WhirlpoolEvmConfig,
}

impl<'a> ActivationSourceResolver<'a> {
    pub fn new(evm_config: &'a WhirlpoolEvmConfig) -> Self {
        Self { evm_config }
    }

    pub fn resolve_players_for_epoch(
        &self,
        target_epoch: u64,
    ) -> Result<Vec<[u8; 32]>, EvmAppError> {
        let _ = target_epoch;
        Ok(self.evm_config.simplex_consensus_public_keys())
    }
}

#[cfg(test)]
mod tests {
    use super::BoundaryEpochContext;

    #[test]
    fn boundary_context_is_forward_looking() {
        let context = BoundaryEpochContext::from_post_advance_epoch(7).expect("context");
        assert_eq!(context.boundary_epoch_e, 7);
        assert_eq!(context.full_dkg_epoch, 8);
        assert_eq!(context.reshare_target_epoch, 9);
    }

    #[test]
    fn boundary_context_rejects_overflow() {
        assert!(BoundaryEpochContext::from_post_advance_epoch(u64::MAX).is_err());
    }
}
