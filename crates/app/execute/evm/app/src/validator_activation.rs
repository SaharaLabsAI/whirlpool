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
        let players = self
            .evm_config
            .activation_players_for_epoch(target_epoch)
            .ok_or_else(|| {
                EvmAppError::InvalidBlock(format!(
                    "activation resolver missing player set for epoch {target_epoch}"
                ))
            })?;
        Ok(players)
    }
}

#[cfg(test)]
mod tests {
    use super::{ActivationSourceResolver, BoundaryEpochContext};
    use crate::config::WhirlpoolEvmConfig;
    use alloy_primitives::Address;
    use chainspec::build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators;
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use validators::ValidatorEntry;

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

    fn sample_config() -> WhirlpoolEvmConfig {
        let validators = vec![ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: Address::repeat_byte(0x11),
        }];
        let chain_spec = Arc::new(
            build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
                BTreeMap::new(),
                BTreeMap::new(),
                validators,
            ),
        );
        WhirlpoolEvmConfig::new(chain_spec)
    }

    #[test]
    fn resolver_uses_epoch_override() {
        let config = sample_config()
            .with_activation_players_for_epoch(2, vec![[0x21; 32]])
            .with_activation_players_for_epoch(3, vec![[0x31; 32], [0x32; 32]]);
        let resolver = ActivationSourceResolver::new(&config);

        assert_eq!(
            resolver
                .resolve_players_for_epoch(2)
                .expect("epoch-2 players should resolve"),
            vec![[0x21; 32]]
        );
        assert_eq!(
            resolver
                .resolve_players_for_epoch(3)
                .expect("epoch-3 players should resolve"),
            vec![[0x31; 32], [0x32; 32]]
        );
    }

    #[test]
    fn resolver_fails_closed_when_epoch_override_missing() {
        let config = sample_config().with_activation_players_for_epoch(2, vec![[0x21; 32]]);
        let resolver = ActivationSourceResolver::new(&config);
        assert!(resolver.resolve_players_for_epoch(3).is_err());
    }
}
