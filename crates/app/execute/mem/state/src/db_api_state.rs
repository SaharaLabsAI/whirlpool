use revm::primitives::B256;

use state::traits::StateDb;

use crate::db::InMemoryStateDb;

impl InMemoryStateDb {
    pub fn state_root(&self) -> B256 {
        <Self as StateDb>::state_root(self).unwrap_or_else(|e| match e {})
    }

    pub fn insert_block_hash(&mut self, number: u64, hash: B256) {
        <Self as StateDb>::insert_block_hash(self, number, hash).unwrap_or_else(|e| match e {})
    }

    pub fn get_block_hash(&self, number: u64) -> B256 {
        <Self as StateDb>::get_block_hash(self, number).unwrap_or_else(|e| match e {})
    }
}
