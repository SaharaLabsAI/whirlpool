use alloy_primitives::{B256, U256};
use revm::primitives::Address;
use revm::state::{AccountInfo, Bytecode};
use revm::DatabaseRef;
use state::StateDb;

use crate::db::RethStateDb;

fn to_state_error(error: impl std::fmt::Display) -> state::StateError {
    state::StateError::Internal(error.to_string())
}

impl revm::DatabaseRef for RethStateDb {
    type Error = state::StateError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        StateDb::get_account(self, address).map_err(to_state_error)
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        StateDb::get_code_by_hash(self, code_hash).map_err(to_state_error)
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        StateDb::get_storage(self, address, index).map_err(to_state_error)
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        StateDb::get_block_hash(self, number).map_err(to_state_error)
    }
}

impl revm::Database for RethStateDb {
    type Error = state::StateError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.basic_ref(address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.code_by_hash_ref(code_hash)
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.storage_ref(address, index)
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.block_hash_ref(number)
    }
}
