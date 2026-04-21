use super::*;

impl WhirlpoolEvmConfig {
    pub fn chain_spec(&self) -> &Arc<ChainSpec> {
        self.inner.chain_spec()
    }
}
