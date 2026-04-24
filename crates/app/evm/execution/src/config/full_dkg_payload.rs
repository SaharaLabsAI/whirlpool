use app::{FullDkgOutputV1, FullDkgV1};

use crate::config::WhirlpoolEvmConfig;

impl WhirlpoolEvmConfig {
    pub fn full_dkg_strict_height(&self) -> u64 {
        self.full_dkg_strict_height
    }

    pub fn with_current_full_dkg_output(mut self, output: FullDkgOutputV1) -> Self {
        self.current_full_dkg_output = Some(output);
        self
    }

    pub fn current_full_dkg_payload(&self, epoch: u64) -> Option<FullDkgV1> {
        let output = self.current_full_dkg_output.clone()?;
        Some(FullDkgV1 { epoch, output })
    }
}
