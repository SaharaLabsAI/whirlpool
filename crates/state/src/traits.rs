use std::collections::HashMap;

use alloy_genesis::GenesisAccount;
use revm::database::BundleState;
use revm::primitives::{Address, B256, U256};
use revm::state::{AccountInfo, Bytecode};

pub trait StateDb {
    fn new() -> Self
    where
        Self: Sized;

    fn with_genesis(alloc: HashMap<Address, GenesisAccount>) -> Self
    where
        Self: Sized;

    fn state_root(&self) -> B256;
    fn commit(&mut self, bundle: &BundleState);
    fn get_account(&self, address: Address) -> Option<AccountInfo>;
    fn get_code_by_hash(&self, code_hash: B256) -> Bytecode;
    fn get_storage(&self, address: Address, index: U256) -> U256;
    fn get_block_hash(&self, number: u64) -> B256;
    fn insert_account(&mut self, address: Address, info: AccountInfo);
    fn insert_block_hash(&mut self, number: u64, hash: B256);
}
