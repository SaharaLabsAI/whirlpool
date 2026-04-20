use alloy_primitives::{Address, B256};
use app::{FullDkgOutputV1, FullDkgV1};
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
    activation_players_by_epoch: BTreeMap<u64, Vec<[u8; 32]>>,
    full_dkg_feature_enabled: bool,
    full_dkg_strict_height: u64,
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
            full_dkg_strict_height: 0,
            current_full_dkg_output: None,
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

    pub fn simplex_consensus_public_keys(&self) -> Vec<[u8; 32]> {
        self.simplex_validators
            .iter()
            .map(|validator| validator.consensus_pubkey)
            .collect()
    }

    pub fn with_activation_players_for_epoch(mut self, epoch: u64, players: Vec<[u8; 32]>) -> Self {
        self.activation_players_by_epoch.insert(epoch, players);
        self
    }

    pub fn activation_players_for_epoch(&self, epoch: u64) -> Option<Vec<[u8; 32]>> {
        if self.activation_players_by_epoch.is_empty() {
            return Some(self.simplex_consensus_public_keys());
        }
        self.activation_players_by_epoch.get(&epoch).cloned()
    }

    pub fn with_full_dkg_feature_enabled(mut self, enabled: bool) -> Self {
        self.full_dkg_feature_enabled = enabled;
        self
    }

    pub fn full_dkg_feature_enabled(&self) -> bool {
        self.full_dkg_feature_enabled
    }

    pub fn with_full_dkg_strict_height(mut self, height: u64) -> Self {
        self.full_dkg_strict_height = height;
        self
    }

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
    use evm_precompiles::{
        COMMUNITY_POOL_ADDRESS, EPOCH_PRECOMPILE_ADDRESS, FEE_POOL_PRECOMPILE_ADDRESS,
        VALIDATORS_PRECOMPILE_ADDRESS,
    };
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

        assert!(evm.precompiles().get(&COMMUNITY_POOL_ADDRESS).is_some());
        assert!(evm
            .precompiles()
            .get(&FEE_POOL_PRECOMPILE_ADDRESS)
            .is_some());
        assert!(evm
            .precompiles()
            .get(&VALIDATORS_PRECOMPILE_ADDRESS)
            .is_some());
        assert!(evm.precompiles().get(&EPOCH_PRECOMPILE_ADDRESS).is_some());
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

    #[test]
    fn test_full_dkg_strict_height_defaults_to_zero() {
        let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
        assert_eq!(config.full_dkg_strict_height(), 0);
    }

    #[test]
    fn activation_players_default_to_simplex_registry() {
        let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
        let expected = config.simplex_consensus_public_keys();
        let resolved = config
            .activation_players_for_epoch(42)
            .expect("default activation players should resolve");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn activation_players_can_be_epoch_overridden() {
        let players_epoch_7 = vec![[0x77; 32], [0x78; 32]];
        let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()))
            .with_activation_players_for_epoch(7, players_epoch_7.clone());

        assert_eq!(
            config.activation_players_for_epoch(7),
            Some(players_epoch_7)
        );
        assert_eq!(config.activation_players_for_epoch(8), None);
    }
}
