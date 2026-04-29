use alloy_genesis::GenesisAccount;
use alloy_primitives::Address;
use reth_chainspec::ChainSpec;
use std::collections::BTreeMap;
use validators_reader::ValidatorEntry;

use crate::{
    try_build_sahara_chain_spec_with_alloc_and_validators_and_community_pool_unlock_config,
    CommunityPoolUnlockConfig, NativeTokenError,
};

pub fn try_build_sahara_chain_spec_with_alloc_and_validators(
    alloc: BTreeMap<Address, GenesisAccount>,
    simplex_validators: Vec<ValidatorEntry>,
) -> Result<ChainSpec, NativeTokenError> {
    try_build_sahara_chain_spec_with_alloc_and_validators_and_community_pool_unlock_config(
        alloc,
        simplex_validators,
        CommunityPoolUnlockConfig::default(),
    )
}

pub fn build_sahara_chain_spec_with_alloc_and_validators_and_community_pool_unlock_config(
    alloc: BTreeMap<Address, GenesisAccount>,
    simplex_validators: Vec<ValidatorEntry>,
    community_pool_unlock_config: CommunityPoolUnlockConfig,
) -> ChainSpec {
    try_build_sahara_chain_spec_with_alloc_and_validators_and_community_pool_unlock_config(
        alloc,
        simplex_validators,
        community_pool_unlock_config,
    )
    .expect("provided genesis alloc should satisfy native-token cap")
}
