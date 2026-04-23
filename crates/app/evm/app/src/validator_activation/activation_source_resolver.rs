use crate::config::WhirlpoolEvmConfig;
use crate::error::EvmAppError;

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
