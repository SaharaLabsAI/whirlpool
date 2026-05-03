mod storage;

use alloy_genesis::Genesis;
use alloy_genesis::GenesisAccount;
use alloy_primitives::Address;
use alloy_primitives::U256;
use reth_chainspec::Chain;
use reth_chainspec::ChainSpec;
use reth_chainspec::ChainSpecBuilder;
use std::collections::BTreeMap;
use validators_reader::ValidatorEntry;

use crate::community_pool::CommunityPoolUnlockConfig;
use crate::native_token::validate_genesis_alloc;
use crate::native_token::NativeTokenError;
use crate::SAHARA_CHAIN_ID;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaharaGenesisConfig {
    pub alloc: BTreeMap<Address, GenesisAccount>,
    pub simplex_validators: Vec<ValidatorEntry>,
    pub community_pool_unlock: CommunityPoolUnlockConfig,
}

impl Default for SaharaGenesisConfig {
    fn default() -> Self {
        Self {
            alloc: BTreeMap::new(),
            simplex_validators: Vec::new(),
            community_pool_unlock: CommunityPoolUnlockConfig::default(),
        }
    }
}

pub fn build_sahara_chain_spec() -> ChainSpec {
    build_sahara_chain_spec_from(SaharaGenesisConfig::default())
}

pub fn build_sahara_chain_spec_from(config: SaharaGenesisConfig) -> ChainSpec {
    try_build_sahara_chain_spec_from(config)
        .expect("provided Sahara genesis config should satisfy native-token cap")
}

pub fn try_build_sahara_chain_spec_from(
    config: SaharaGenesisConfig,
) -> Result<ChainSpec, NativeTokenError> {
    let SaharaGenesisConfig {
        mut alloc,
        simplex_validators,
        community_pool_unlock,
    } = config;

    if community_pool_unlock.is_unlock_enabled() && simplex_validators.is_empty() {
        return Err(NativeTokenError::CommunityPoolUnlockRequiresValidators);
    }

    storage::seed_validator_registry(&mut alloc, &simplex_validators);
    storage::seed_epoch_precompile_genesis_state(&mut alloc);
    storage::seed_community_pool_genesis_state(&mut alloc, community_pool_unlock)?;

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
