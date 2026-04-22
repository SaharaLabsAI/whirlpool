use super::*;

impl WhirlpoolEvmConfig {
    pub fn with_epoch_boundary_hook(mut self, hook: EpochBoundaryHook) -> Self {
        self.epoch_boundary_hook = hook;
        self
    }

    pub fn epoch_boundary_hook(&self) -> EpochBoundaryHook {
        self.epoch_boundary_hook
    }
}
