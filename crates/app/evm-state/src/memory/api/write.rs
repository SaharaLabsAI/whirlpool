use revm::database::BundleState;
use revm::primitives::{Address, U256};
use revm::state::AccountInfo;

use state::traits::StateDb;

use crate::memory::db::InMemoryStateDb;

impl InMemoryStateDb {
    pub fn commit(&mut self, bundle: &BundleState) {
        <Self as StateDb>::commit(self, bundle).unwrap_or_else(|e| match e {})
    }

    pub fn insert_account(&mut self, address: Address, info: AccountInfo) {
        <Self as StateDb>::insert_account(self, address, info).unwrap_or_else(|e| match e {})
    }

    pub fn insert_storage(&mut self, address: Address, index: U256, value: U256) {
        <Self as StateDb>::insert_storage(self, address, index, value)
            .unwrap_or_else(|e| match e {})
    }
}
