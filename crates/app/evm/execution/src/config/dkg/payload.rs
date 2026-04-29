use validators_dkg::FullDkgOutputV1;

use crate::config::WhirlpoolEvmConfig;

impl WhirlpoolEvmConfig {
    pub fn with_current_full_dkg_output(mut self, output: FullDkgOutputV1) -> Self {
        self.current_full_dkg_output = Some(output);
        self
    }

    pub fn current_full_dkg_output(&self) -> Option<&FullDkgOutputV1> {
        self.current_full_dkg_output.as_ref()
    }
}
