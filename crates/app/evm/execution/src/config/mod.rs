use core::convert::Infallible;
use evm_precompiles::{whirlpool_precompiles, WhirlpoolEvmFactory};
use reth_chainspec::ChainSpec;
use reth_ethereum_primitives::EthPrimitives;
use reth_evm::{
    eth::EthEvmBuilder, ConfigureEvm, EvmEnvFor, EvmFor, ExecutionCtxFor, NextBlockEnvAttributes,
};
use reth_evm_ethereum::EthEvmConfig;
use reth_primitives_traits::{BlockTy, HeaderTy, SealedBlock, SealedHeader};
use std::collections::BTreeMap;
use std::sync::Arc;
use validators_dkg::FullDkgOutputV1;

mod chain_spec_access;
mod dkg;
mod proposer;

type WhirlpoolInnerEvmConfig = EthEvmConfig<ChainSpec, WhirlpoolEvmFactory>;

#[derive(Debug, Clone)]
pub struct WhirlpoolEvmConfig {
    inner: WhirlpoolInnerEvmConfig,
    local_proposer_public_key: [u8; 32],
    activation_players_by_epoch: BTreeMap<u64, Vec<[u8; 32]>>,
    full_dkg_feature_enabled: bool,
    current_full_dkg_output: Option<FullDkgOutputV1>,
}

impl WhirlpoolEvmConfig {
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self {
            inner: EthEvmConfig::new_with_evm_factory(chain_spec, WhirlpoolEvmFactory),
            local_proposer_public_key: [0u8; 32],
            activation_players_by_epoch: BTreeMap::new(),
            full_dkg_feature_enabled: true,
            current_full_dkg_output: None,
        }
    }

    pub fn with_local_proposer_public_key(mut self, local_proposer_public_key: [u8; 32]) -> Self {
        self.local_proposer_public_key = local_proposer_public_key;
        self
    }
}

impl ConfigureEvm for WhirlpoolEvmConfig {
    type Primitives = EthPrimitives;
    type Error = Infallible;
    type NextBlockEnvCtx = NextBlockEnvAttributes;
    type BlockExecutorFactory = <WhirlpoolInnerEvmConfig as ConfigureEvm>::BlockExecutorFactory;
    type BlockAssembler = <WhirlpoolInnerEvmConfig as ConfigureEvm>::BlockAssembler;

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
mod tests;
