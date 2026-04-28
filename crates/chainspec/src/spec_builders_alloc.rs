use alloy_genesis::GenesisAccount;
use alloy_primitives::Address;
use reth_chainspec::ChainSpec;
use std::collections::BTreeMap;
use validators_reader::ValidatorEntry;

use crate::{
    try_build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config,
    CommunityPoolUnlockConfig, NativeTokenError,
};

pub fn try_build_sahara_chain_spec_with_alloc(
    alloc: BTreeMap<Address, GenesisAccount>,
) -> Result<ChainSpec, NativeTokenError> {
    try_build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config(
        alloc,
        BTreeMap::new(),
        Vec::new(),
        CommunityPoolUnlockConfig::default(),
    )
}

pub fn build_sahara_chain_spec_with_alloc_and_fee_recipients(
    alloc: BTreeMap<Address, GenesisAccount>,
    validator_fee_recipients: BTreeMap<[u8; 32], Address>,
) -> ChainSpec {
    try_build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config(
        alloc,
        validator_fee_recipients,
        Vec::new(),
        CommunityPoolUnlockConfig::default(),
    )
    .expect("provided genesis alloc should satisfy native-token cap")
}

pub fn build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
    alloc: BTreeMap<Address, GenesisAccount>,
    validator_fee_recipients: BTreeMap<[u8; 32], Address>,
    simplex_validators: Vec<ValidatorEntry>,
) -> ChainSpec {
    try_build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config(
        alloc,
        validator_fee_recipients,
        simplex_validators,
        CommunityPoolUnlockConfig::default(),
    )
    .expect("provided genesis alloc should satisfy native-token cap")
}
