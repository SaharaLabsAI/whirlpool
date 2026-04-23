use revm::primitives::{Address, B256, U256};
use revm::state::{AccountInfo, Bytecode};

use state::traits::StateDb;

use crate::in_memory_db::InMemoryStateDb;

impl InMemoryStateDb {
    pub fn get_account(&self, address: Address) -> Option<AccountInfo> {
        <Self as StateDb>::get_account(self, address).unwrap_or_else(|e| match e {})
    }

    pub fn get_code_by_hash(&self, code_hash: B256) -> Bytecode {
        <Self as StateDb>::get_code_by_hash(self, code_hash).unwrap_or_else(|e| match e {})
    }

    pub fn get_storage(&self, address: Address, index: U256) -> U256 {
        <Self as StateDb>::get_storage(self, address, index).unwrap_or_else(|e| match e {})
    }
}
