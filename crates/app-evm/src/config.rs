use alloy_genesis::{Genesis, GenesisAccount};
use alloy_primitives::{Address, B256, U256};
use core::convert::Infallible;
use native_token::{validate_genesis_alloc, NativeTokenError};
use reth_chainspec::{Chain, ChainSpec, ChainSpecBuilder};
use reth_ethereum_primitives::EthPrimitives;
use reth_evm::{ConfigureEvm, EvmEnvFor, ExecutionCtxFor, NextBlockEnvAttributes};
use reth_evm_ethereum::EthEvmConfig;
use reth_primitives_traits::{BlockTy, HeaderTy, SealedBlock, SealedHeader};
use std::collections::BTreeMap;
use std::sync::Arc;

pub const SAHARA_CHAIN_ID: u64 = 313_371;
pub const DEFAULT_PROPOSER_FEE_RECIPIENT: Address = Address::new([
    0x70, 0x72, 0x6f, 0x70, 0x6f, 0x73, 0x65, 0x72, 0x2d, 0x66, 0x65, 0x65, 0x2d, 0x73, 0x65, 0x61,
    0x6d, 0x2d, 0x30, 0x31,
]);
pub const VALIDATOR_FEE_RECIPIENTS_REGISTRY: Address = Address::new([
    0x76, 0x61, 0x6c, 0x69, 0x64, 0x61, 0x74, 0x6f, 0x72, 0x2d, 0x66, 0x65, 0x65, 0x2d, 0x6d, 0x61,
    0x70, 0x2d, 0x30, 0x31,
]);

pub fn build_sahara_chain_spec() -> ChainSpec {
    try_build_sahara_chain_spec()
        .expect("default Sahara chain spec should satisfy native-token cap")
}

pub fn try_build_sahara_chain_spec() -> Result<ChainSpec, NativeTokenError> {
    try_build_sahara_chain_spec_with_alloc(BTreeMap::new())
}

/// Build the Sahara chain spec with pre-funded genesis accounts.
///
/// This is useful for integration tests that need accounts with ETH balances
/// at genesis to submit transactions.
pub fn build_sahara_chain_spec_with_alloc(alloc: BTreeMap<Address, GenesisAccount>) -> ChainSpec {
    try_build_sahara_chain_spec_with_alloc(alloc)
        .expect("provided genesis alloc should satisfy native-token cap")
}

pub fn try_build_sahara_chain_spec_with_alloc(
    alloc: BTreeMap<Address, GenesisAccount>,
) -> Result<ChainSpec, NativeTokenError> {
    try_build_sahara_chain_spec_with_alloc_and_fee_recipients(alloc, BTreeMap::new())
}

pub fn build_sahara_chain_spec_with_alloc_and_fee_recipients(
    alloc: BTreeMap<Address, GenesisAccount>,
    validator_fee_recipients: BTreeMap<[u8; 32], Address>,
) -> ChainSpec {
    try_build_sahara_chain_spec_with_alloc_and_fee_recipients(alloc, validator_fee_recipients)
        .expect("provided genesis alloc should satisfy native-token cap")
}

pub fn try_build_sahara_chain_spec_with_alloc_and_fee_recipients(
    mut alloc: BTreeMap<Address, GenesisAccount>,
    validator_fee_recipients: BTreeMap<[u8; 32], Address>,
) -> Result<ChainSpec, NativeTokenError> {
    if !validator_fee_recipients.is_empty() {
        let account = alloc
            .entry(VALIDATOR_FEE_RECIPIENTS_REGISTRY)
            .or_insert_with(|| GenesisAccount {
                balance: U256::ZERO,
                ..GenesisAccount::default()
            });

        let storage = account.storage.get_or_insert_with(BTreeMap::new);
        for (validator_public_key, fee_recipient) in validator_fee_recipients {
            storage.insert(
                B256::from(validator_public_key),
                fee_recipient_storage_value(fee_recipient),
            );
        }
    }

    validate_genesis_alloc(&alloc)?;

    Ok(ChainSpecBuilder::default()
        .chain(Chain::from_id(SAHARA_CHAIN_ID))
        .genesis(Genesis {
            gas_limit: 30_000_000,
            difficulty: U256::ZERO,
            alloc,
            ..Default::default()
        })
        .cancun_activated()
        .build())
}

#[derive(Debug, Clone)]
pub struct WhirlpoolEvmConfig {
    inner: EthEvmConfig,
    local_proposer_public_key: [u8; 32],
    validator_fee_recipients: BTreeMap<[u8; 32], Address>,
}

impl WhirlpoolEvmConfig {
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        let validator_fee_recipients = validator_fee_recipients_from_chain_spec(&chain_spec);
        Self {
            inner: EthEvmConfig::new(chain_spec),
            local_proposer_public_key: [0u8; 32],
            validator_fee_recipients,
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
}

fn fee_recipient_storage_value(fee_recipient: Address) -> B256 {
    let mut bytes = [0u8; 32];
    bytes[12..].copy_from_slice(fee_recipient.as_slice());
    B256::from(bytes)
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
    use super::{
        build_sahara_chain_spec, build_sahara_chain_spec_with_alloc_and_fee_recipients,
        try_build_sahara_chain_spec_with_alloc, WhirlpoolEvmConfig, DEFAULT_PROPOSER_FEE_RECIPIENT,
        SAHARA_CHAIN_ID, VALIDATOR_FEE_RECIPIENTS_REGISTRY,
    };
    use alloy_genesis::GenesisAccount;
    use alloy_primitives::{Address, U256};
    use native_token::{sahara_hard_cap_base_units, NativeTokenError};
    use reth_chainspec::EthereumHardforks;
    use reth_evm::ConfigureEvm;
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
    fn test_build_sahara_chain_spec_values() {
        let spec = build_sahara_chain_spec();

        assert_eq!(spec.chain.id(), SAHARA_CHAIN_ID);
        assert_eq!(spec.genesis.gas_limit, 30_000_000);
        assert!(spec.is_cancun_active_at_timestamp(0));
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
    fn test_try_build_sahara_chain_spec_with_alloc_rejects_over_cap() {
        let mut alloc = BTreeMap::new();
        let total = sahara_hard_cap_base_units() + U256::from(1u64);
        alloc.insert(
            Address::repeat_byte(0x55),
            GenesisAccount {
                balance: total,
                ..GenesisAccount::default()
            },
        );

        assert_eq!(
            try_build_sahara_chain_spec_with_alloc(alloc),
            Err(NativeTokenError::HardCapExceeded {
                total,
                hard_cap: sahara_hard_cap_base_units(),
            })
        );
    }
}
