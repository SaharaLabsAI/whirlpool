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
use validators::{
    decode_validator_registry_storage_opt, ValidatorEntry, ValidatorRegistryError,
    SIMPLEX_VALIDATORS_REGISTRY,
};

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
        }
    }

    pub fn with_local_proposer_public_key(mut self, local_proposer_public_key: [u8; 32]) -> Self {
        self.local_proposer_public_key = local_proposer_public_key;
        self
    }

    pub fn chain_spec(&self) -> &Arc<ChainSpec> {
        self.inner.chain_spec()
    }

    pub fn fee_recipient(&self) -> Address {
        self.fee_recipient_for_proposer(self.local_proposer_public_key)
            .unwrap_or(DEFAULT_PROPOSER_FEE_RECIPIENT)
    }

    pub fn fee_recipient_for_proposer(&self, proposer_public_key: [u8; 32]) -> Option<Address> {
        self.validator_fee_recipients
            .get(&proposer_public_key)
            .copied()
    }

    pub fn local_proposer_public_key(&self) -> [u8; 32] {
        self.local_proposer_public_key
    }

    pub fn simplex_validators(&self) -> &[ValidatorEntry] {
        &self.simplex_validators
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
mod tests {
    use super::{
        WhirlpoolEvmConfig, DEFAULT_PROPOSER_FEE_RECIPIENT, VALIDATOR_FEE_RECIPIENTS_REGISTRY,
    };
    use alloy_primitives::Address;
    use chainspec::{
        build_sahara_chain_spec, build_sahara_chain_spec_with_alloc_and_fee_recipients,
        SAHARA_CHAIN_ID,
    };
    use evm_precompiles::{TEST_TOKEN_PRECOMPILE_ADDRESS, VALIDATORS_PRECOMPILE_ADDRESS};
    use reth_chainspec::EthereumHardforks;
    use reth_evm::{ConfigureEvm, Evm, EvmFactory, NextBlockEnvAttributes};
    use reth_primitives_traits::Header;
    use revm::{database::EmptyDB, primitives::B256};
    use std::collections::BTreeMap;
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
    fn test_evm_config_installs_whirlpool_precompiles() {
        let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
        let env = config
            .next_evm_env(
                &Header::default(),
                &NextBlockEnvAttributes {
                    timestamp: 1,
                    suggested_fee_recipient: Address::ZERO,
                    prev_randao: B256::ZERO,
                    gas_limit: 30_000_000,
                    parent_beacon_block_root: Some(B256::ZERO),
                    withdrawals: None,
                    extra_data: Default::default(),
                },
            )
            .expect("next EVM env");
        let evm = config.evm_factory().create_evm(EmptyDB::default(), env);

        assert!(evm
            .precompiles()
            .get(&TEST_TOKEN_PRECOMPILE_ADDRESS)
            .is_some());
        assert!(evm
            .precompiles()
            .get(&VALIDATORS_PRECOMPILE_ADDRESS)
            .is_some());
    }

    #[test]
    fn test_default_fee_recipient_is_non_zero() {
        let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));

        assert_eq!(config.fee_recipient(), DEFAULT_PROPOSER_FEE_RECIPIENT);
        assert_ne!(config.fee_recipient(), Address::ZERO);
    }

    #[test]
    fn test_fee_recipient_mapping_roundtrip_in_genesis_registry() {
        let local_proposer_public_key = [0x11; 32];
        let custom = Address::repeat_byte(0x44);
        let mut validator_fee_recipients = BTreeMap::new();
        validator_fee_recipients.insert(local_proposer_public_key, custom);

        let spec = Arc::new(build_sahara_chain_spec_with_alloc_and_fee_recipients(
            BTreeMap::new(),
            validator_fee_recipients,
        ));
        let config = WhirlpoolEvmConfig::new(spec.clone())
            .with_local_proposer_public_key(local_proposer_public_key);

        assert_eq!(config.fee_recipient(), custom);
        assert!(spec
            .genesis
            .alloc
            .contains_key(&VALIDATOR_FEE_RECIPIENTS_REGISTRY));
    }
}
