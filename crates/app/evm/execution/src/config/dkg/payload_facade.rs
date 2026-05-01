use validators_dkg::FullDkgOutputV1;

use crate::config::WhirlpoolEvmConfig;

impl WhirlpoolEvmConfig {
    pub fn with_current_full_dkg_output(mut self, output: FullDkgOutputV1) -> Self {
        self.dkg_transition = self.dkg_transition.with_current_full_dkg_output(output);
        self
    }

    pub fn current_full_dkg_output(&self) -> Option<&FullDkgOutputV1> {
        self.dkg_transition.current_candidate().output()
    }
}
