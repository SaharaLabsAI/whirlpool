use alloy_genesis::GenesisAccount;
use alloy_primitives::Address;
use reth_chainspec::ChainSpec;
use std::collections::BTreeMap;

use crate::{try_build_sahara_chain_spec_with_alloc, NativeTokenError};

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
