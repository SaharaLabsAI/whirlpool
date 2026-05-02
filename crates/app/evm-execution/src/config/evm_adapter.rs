use core::convert::Infallible;
use std::sync::Arc;

use evm_precompiles::{whirlpool_precompiles, WhirlpoolEvmFactory};
use reth_chainspec::ChainSpec;
use reth_ethereum_primitives::EthPrimitives;
use reth_evm::{
    eth::EthEvmBuilder, ConfigureEvm, EvmEnvFor, EvmFor, ExecutionCtxFor, NextBlockEnvAttributes,
};
use reth_evm_ethereum::EthEvmConfig;
use reth_primitives_traits::{BlockTy, HeaderTy, SealedBlock, SealedHeader};

use crate::config::WhirlpoolEvmConfig;

type WhirlpoolInnerEvmConfig = EthEvmConfig<ChainSpec, WhirlpoolEvmFactory>;

#[derive(Debug, Clone)]
pub struct WhirlpoolRethEvmAdapter {
    reth_config: WhirlpoolInnerEvmConfig,
}

impl WhirlpoolRethEvmAdapter {
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self {
            reth_config: EthEvmConfig::new_with_evm_factory(chain_spec, WhirlpoolEvmFactory),
        }
    }

    pub fn chain_spec(&self) -> &Arc<ChainSpec> {
        self.reth_config.chain_spec()
    }
}

impl ConfigureEvm for WhirlpoolEvmConfig {
    type Primitives = EthPrimitives;
    type Error = Infallible;
    type NextBlockEnvCtx = NextBlockEnvAttributes;
    type BlockExecutorFactory = <WhirlpoolInnerEvmConfig as ConfigureEvm>::BlockExecutorFactory;
    type BlockAssembler = <WhirlpoolInnerEvmConfig as ConfigureEvm>::BlockAssembler;

    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        self.evm_adapter.reth_config.block_executor_factory()
    }

    fn block_assembler(&self) -> &Self::BlockAssembler {
        self.evm_adapter.reth_config.block_assembler()
    }

    fn evm_env(&self, header: &HeaderTy<Self::Primitives>) -> Result<EvmEnvFor<Self>, Self::Error> {
        self.evm_adapter.reth_config.evm_env(header)
    }

    fn next_evm_env(
        &self,
        parent: &HeaderTy<Self::Primitives>,
        attributes: &Self::NextBlockEnvCtx,
    ) -> Result<EvmEnvFor<Self>, Self::Error> {
        self.evm_adapter
            .reth_config
            .next_evm_env(parent, attributes)
    }

    fn evm_with_env<DB: reth_evm::Database>(
        &self,
        db: DB,
        evm_env: EvmEnvFor<Self>,
    ) -> EvmFor<Self, DB> {
        let spec = evm_env.cfg_env.spec;
        EthEvmBuilder::new(db, evm_env)
            .precompiles(whirlpool_precompiles(spec))
            .build()
    }

    fn context_for_block<'a>(
        &self,
        block: &'a SealedBlock<BlockTy<Self::Primitives>>,
    ) -> Result<ExecutionCtxFor<'a, Self>, Self::Error> {
        self.evm_adapter.reth_config.context_for_block(block)
    }

    fn context_for_next_block(
        &self,
        parent: &SealedHeader<HeaderTy<Self::Primitives>>,
        attributes: Self::NextBlockEnvCtx,
    ) -> Result<ExecutionCtxFor<'_, Self>, Self::Error> {
        self.evm_adapter
            .reth_config
            .context_for_next_block(parent, attributes)
    }
}
