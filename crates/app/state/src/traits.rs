use std::collections::HashMap;

use alloy_genesis::GenesisAccount;
use revm::database::BundleState;
use revm::primitives::{Address, B256, U256};
use revm::state::{AccountInfo, Bytecode};

pub trait StateDb {
    type Error: std::error::Error + Send + Sync + 'static;

    fn new() -> Self
    where
        Self: Sized;

    fn with_genesis(alloc: HashMap<Address, GenesisAccount>) -> Self
    where
        Self: Sized;

    fn state_root(&self) -> Result<B256, Self::Error>;
    fn commit(&mut self, bundle: &BundleState) -> Result<(), Self::Error>;
    fn get_account(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;
    fn get_code_by_hash(&self, code_hash: B256) -> Result<Bytecode, Self::Error>;
    fn get_storage(&self, address: Address, index: U256) -> Result<U256, Self::Error>;
    fn get_block_hash(&self, number: u64) -> Result<B256, Self::Error>;
    fn insert_account(&mut self, address: Address, info: AccountInfo) -> Result<(), Self::Error>;
    fn insert_block_hash(&mut self, number: u64, hash: B256) -> Result<(), Self::Error>;
}
