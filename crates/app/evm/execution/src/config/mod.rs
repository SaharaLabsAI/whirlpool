use alloy_primitives::{Address, B256};
use core::convert::Infallible;
use evm_precompiles::{whirlpool_precompiles_with_validators, WhirlpoolEvmFactory};
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
use validators_reader::{
    decode_validator_registry_storage_opt, ValidatorEntry, ValidatorRegistryError,
    SIMPLEX_VALIDATORS_REGISTRY,
};

mod activation_players;
mod chain_spec_access;
mod fee_recipients;
mod full_dkg_flags;
mod full_dkg_payload;
mod simplex;

pub const DEFAULT_PROPOSER_FEE_RECIPIENT: Address = Address::new([
    0x70, 0x72, 0x6f, 0x70, 0x6f, 0x73, 0x65, 0x72, 0x2d, 0x66, 0x65, 0x65, 0x2d, 0x73, 0x65, 0x61,
    0x6d, 0x2d, 0x30, 0x31,
]);
pub const VALIDATOR_FEE_RECIPIENTS_REGISTRY: Address = Address::new([
    0x76, 0x61, 0x6c, 0x69, 0x64, 0x61, 0x74, 0x6f, 0x72, 0x2d, 0x66, 0x65, 0x65, 0x2d, 0x6d, 0x61,
    0x70, 0x2d, 0x30, 0x31,
]);

type WhirlpoolInnerEvmConfig = EthEvmConfig<ChainSpec, WhirlpoolEvmFactory>;

#[derive(Debug, Clone)]
pub struct WhirlpoolEvmConfig {
    inner: WhirlpoolInnerEvmConfig,
    local_proposer_public_key: [u8; 32],
    validator_fee_recipients: BTreeMap<[u8; 32], Address>,
    simplex_validators: Vec<ValidatorEntry>,
    activation_players_by_epoch: BTreeMap<u64, Vec<[u8; 32]>>,
    full_dkg_feature_enabled: bool,
    current_full_dkg_output: Option<FullDkgOutputV1>,
}

impl WhirlpoolEvmConfig {
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        let validator_fee_recipients = validator_fee_recipients_from_chain_spec(&chain_spec);
        let simplex_validators = simplex_validators_from_chain_spec(&chain_spec)
            .expect("simplex validators registry encoding should decode");
        Self {
            inner: EthEvmConfig::new_with_evm_factory(
                chain_spec,
                WhirlpoolEvmFactory::with_validators(simplex_validators.clone()),
            ),
            local_proposer_public_key: [0u8; 32],
            validator_fee_recipients,
            simplex_validators,
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

fn fee_recipient_from_storage_value(value: B256) -> Address {
    Address::from_slice(&value.as_slice()[12..])
}

fn validator_fee_recipients_from_chain_spec(chain_spec: &ChainSpec) -> BTreeMap<[u8; 32], Address> {
    chain_spec
        .genesis
        .alloc
        .get(&VALIDATOR_FEE_RECIPIENTS_REGISTRY)
        .and_then(|account| account.storage.as_ref())
        .map(|storage| {
            storage
                .iter()
                .map(|(validator_public_key, fee_recipient)| {
                    (
                        validator_public_key.0,
                        fee_recipient_from_storage_value(*fee_recipient),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn simplex_validators_from_chain_spec(
    chain_spec: &ChainSpec,
) -> Result<Vec<ValidatorEntry>, ValidatorRegistryError> {
    decode_validator_registry_storage_opt(
        chain_spec
            .genesis
            .alloc
            .get(&SIMPLEX_VALIDATORS_REGISTRY)
            .and_then(|account| account.storage.as_ref()),
    )
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
            .precompiles(whirlpool_precompiles_with_validators(
                spec,
                self.simplex_validators.clone(),
            ))
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
