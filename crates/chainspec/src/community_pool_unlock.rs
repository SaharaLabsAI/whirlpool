use alloy_primitives::U256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommunityPoolUnlockConfig {
    pub genesis_prefund_amount: U256,
    pub unlock_every_epochs: u64,
    pub unlock_amount_per_cycle: U256,
}

impl CommunityPoolUnlockConfig {
    pub const fn disabled() -> Self {
        Self {
            genesis_prefund_amount: U256::ZERO,
            unlock_every_epochs: 0,
            unlock_amount_per_cycle: U256::ZERO,
        }
    }

    pub fn is_unlock_enabled(&self) -> bool {
        self.unlock_every_epochs > 0 && !self.unlock_amount_per_cycle.is_zero()
    }
}

impl Default for CommunityPoolUnlockConfig {
    fn default() -> Self {
        Self::disabled()
    }
}
