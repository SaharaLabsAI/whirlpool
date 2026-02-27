use alloy_genesis::Genesis;
use alloy_primitives::U256;
use core::convert::Infallible;
use reth_chainspec::{Chain, ChainSpec, ChainSpecBuilder};
use reth_ethereum_primitives::EthPrimitives;
use reth_evm::{ConfigureEvm, EvmEnvFor, ExecutionCtxFor, NextBlockEnvAttributes};
use reth_evm_ethereum::EthEvmConfig;
use reth_primitives_traits::{BlockTy, HeaderTy, SealedBlock, SealedHeader};
use std::sync::Arc;

pub const SAHARA_CHAIN_ID: u64 = 313_371;

pub fn build_sahara_chain_spec() -> ChainSpec {
    ChainSpecBuilder::default()
        .chain(Chain::from_id(SAHARA_CHAIN_ID))
        .genesis(Genesis {
            gas_limit: 30_000_000,
            difficulty: U256::ZERO,
            ..Default::default()
        })
        .cancun_activated()
        .build()
}

#[derive(Debug, Clone)]
pub struct WhirlpoolEvmConfig {
    inner: EthEvmConfig,
}

impl WhirlpoolEvmConfig {
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self { inner: EthEvmConfig::new(chain_spec) }
    }

    pub fn chain_spec(&self) -> &Arc<ChainSpec> {
        self.inner.chain_spec()
    }
}

impl ConfigureEvm for WhirlpoolEvmConfig {
    type Primitives = EthPrimitives;
    type Error = Infallible;
    type NextBlockEnvCtx = NextBlockEnvAttributes;
    type BlockExecutorFactory = <EthEvmConfig as ConfigureEvm>::BlockExecutorFactory;
    type BlockAssembler = <EthEvmConfig as ConfigureEvm>::BlockAssembler;

    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        self.inner.block_executor_factory()
    }

    fn block_assembler(&self) -> &Self::BlockAssembler {
        self.inner.block_assembler()
    }

    fn evm_env(&self, header: &HeaderTy<Self::Primitives>) -> Result<EvmEnvFor<Self>, Self::Error> {
        self.inner.evm_env(header)
    }

    fn next_evm_env(
        &self,
        parent: &HeaderTy<Self::Primitives>,
        attributes: &Self::NextBlockEnvCtx,
    ) -> Result<EvmEnvFor<Self>, Self::Error> {
        self.inner.next_evm_env(parent, attributes)
    }

    fn context_for_block<'a>(
        &self,
        block: &'a SealedBlock<BlockTy<Self::Primitives>>,
    ) -> Result<ExecutionCtxFor<'a, Self>, Self::Error> {
        self.inner.context_for_block(block)
    }

    fn context_for_next_block(
        &self,
        parent: &SealedHeader<HeaderTy<Self::Primitives>>,
        attributes: Self::NextBlockEnvCtx,
    ) -> Result<ExecutionCtxFor<'_, Self>, Self::Error> {
        self.inner.context_for_next_block(parent, attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::{build_sahara_chain_spec, WhirlpoolEvmConfig, SAHARA_CHAIN_ID};
    use reth_chainspec::EthereumHardforks;
    use reth_evm::ConfigureEvm;
    use std::sync::Arc;
    #[test]
    fn test_evm_config_chain_spec() {
        let spec = Arc::new(build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(spec.clone());

        assert!(Arc::ptr_eq(config.chain_spec(), &spec));
        assert_eq!(config.chain_spec().chain.id(), SAHARA_CHAIN_ID);
        assert_eq!(config.chain_spec().genesis.gas_limit, 30_000_000);
        assert!(config.chain_spec().is_cancun_active_at_timestamp(0));
    }

    #[test]
    fn test_evm_config_exposes_factory_and_assembler() {
        let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));

        let _factory: &<WhirlpoolEvmConfig as ConfigureEvm>::BlockExecutorFactory =
            config.block_executor_factory();
        let _assembler: &<WhirlpoolEvmConfig as ConfigureEvm>::BlockAssembler =
            config.block_assembler();
    }

    #[test]
    fn test_build_sahara_chain_spec_values() {
        let spec = build_sahara_chain_spec();

        assert_eq!(spec.chain.id(), SAHARA_CHAIN_ID);
        assert_eq!(spec.genesis.gas_limit, 30_000_000);
        assert!(spec.is_cancun_active_at_timestamp(0));
    }

}