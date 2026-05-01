use crate::config::WhirlpoolEvmConfig;

impl WhirlpoolEvmConfig {
    pub fn with_full_dkg_feature_enabled(mut self, enabled: bool) -> Self {
        self.dkg_transition = self.dkg_transition.with_full_dkg_feature_enabled(enabled);
        self
    }

    pub fn full_dkg_feature_enabled(&self) -> bool {
        self.dkg_transition.feature_gate().enabled()
    }
}
