use std::collections::HashMap;

use alloy_genesis::GenesisAccount;
use revm::database::BundleState;
use revm::primitives::{keccak256, Address, B256, KECCAK_EMPTY, U256};
use revm::state::{AccountInfo, Bytecode};
use revm::{Database, DatabaseRef};

use state::error::StateError;
use state::traits::StateDb;

#[derive(Clone, Debug, Default)]
pub struct DbAccount {
    pub info: AccountInfo,
    pub storage: HashMap<U256, U256>,
}

#[derive(Clone, Debug)]
pub struct InMemoryStateDb {
    accounts: HashMap<Address, DbAccount>,
    bytecodes: HashMap<B256, Bytecode>,
    block_hashes: HashMap<u64, B256>,
}

impl Default for InMemoryStateDb {
    fn default() -> Self {
        Self::new()
    }
}

impl StateDb for InMemoryStateDb {
    type Error = core::convert::Infallible;

    fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            bytecodes: HashMap::new(),
            block_hashes: HashMap::new(),
        }
    }

    fn with_genesis(alloc: HashMap<Address, GenesisAccount>) -> Self {
        let mut db = Self::new();

        for (address, account) in alloc {
            let mut info = AccountInfo {
                balance: account.balance,
                nonce: account.nonce.unwrap_or_default(),
                ..AccountInfo::default()
            };

            if let Some(code_bytes) = account.code {
                let code = Bytecode::new_raw(code_bytes);
                let code_hash = code.hash_slow();
                info.code_hash = code_hash;
                info.code = Some(code.clone());
                db.bytecodes.insert(code_hash, code);
            }

            let mut storage = HashMap::new();
            if let Some(genesis_storage) = account.storage {
                for (key, value) in genesis_storage {
                    storage.insert(U256::from_be_bytes(key.0), U256::from_be_bytes(value.0));
                }
            }

            db.accounts.insert(address, DbAccount { info, storage });
        }

        db
    }

    fn state_root(&self) -> Result<B256, Self::Error> {
        if self.accounts.is_empty() {
            return Ok(KECCAK_EMPTY);
        }

        let mut account_items: Vec<_> = self.accounts.iter().collect();
        account_items.sort_by_key(|(address, _)| *address);

        let mut encoded = Vec::new();
        for (address, account) in account_items {
            encoded.extend_from_slice(address.as_slice());
            encoded.extend_from_slice(&account.info.nonce.to_be_bytes());
            encoded.extend_from_slice(&account.info.balance.to_be_bytes::<32>());
            encoded.extend_from_slice(account.info.code_hash.as_slice());

            let mut storage_items: Vec<_> = account.storage.iter().collect();
            storage_items.sort_by_key(|(key, _)| **key);
            for (key, value) in storage_items {
                encoded.extend_from_slice(&key.to_be_bytes::<32>());
                encoded.extend_from_slice(&value.to_be_bytes::<32>());
            }
        }

        Ok(keccak256(encoded))
    }

    fn commit(&mut self, bundle: &BundleState) -> Result<(), Self::Error> {
        for (address, bundle_account) in &bundle.state {
            if bundle_account.was_destroyed() {
                self.accounts.remove(address);
                continue;
            }

            let Some(info) = bundle_account.account_info() else {
                self.accounts.remove(address);
                continue;
            };

            let account = self.accounts.entry(*address).or_default();
            account.info = info;

            if bundle_account.status.is_storage_known() {
                account.storage.clear();
            }

            for (key, slot) in &bundle_account.storage {
                let value = slot.present_value();
                if value.is_zero() {
                    account.storage.remove(key);
                } else {
                    account.storage.insert(*key, value);
                }
            }
        }

        for (code_hash, bytecode) in &bundle.contracts {
            self.bytecodes.insert(*code_hash, bytecode.clone());
        }

        Ok(())
    }

    fn get_account(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(self
            .accounts
            .get(&address)
            .map(|account| account.info.clone()))
    }

    fn get_code_by_hash(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(self.bytecodes.get(&code_hash).cloned().unwrap_or_default())
    }

    fn get_storage(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Ok(self
            .accounts
            .get(&address)
            .and_then(|account| account.storage.get(&index).copied())
            .unwrap_or(U256::ZERO))
    }

    fn get_block_hash(&self, number: u64) -> Result<B256, Self::Error> {
        Ok(self
            .block_hashes
            .get(&number)
            .copied()
            .unwrap_or(B256::ZERO))
    }

    fn insert_account(&mut self, address: Address, info: AccountInfo) -> Result<(), Self::Error> {
        self.accounts.insert(
            address,
            DbAccount {
                info,
                storage: HashMap::new(),
            },
        );
        Ok(())
    }

    fn insert_storage(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Result<(), Self::Error> {
        let account = self.accounts.entry(address).or_default();
        if value.is_zero() {
            account.storage.remove(&index);
        } else {
            account.storage.insert(index, value);
        }
        Ok(())
    }

    fn insert_block_hash(&mut self, number: u64, hash: B256) -> Result<(), Self::Error> {
        self.block_hashes.insert(number, hash);
        Ok(())
    }
}

impl InMemoryStateDb {
    pub fn new() -> Self {
        <Self as StateDb>::new()
    }

    pub fn with_genesis(alloc: HashMap<Address, GenesisAccount>) -> Self {
        <Self as StateDb>::with_genesis(alloc)
    }

    pub fn commit(&mut self, bundle: &BundleState) {
        <Self as StateDb>::commit(self, bundle).unwrap_or_else(|e| match e {})
    }

    pub fn state_root(&self) -> B256 {
        <Self as StateDb>::state_root(self).unwrap_or_else(|e| match e {})
    }

    pub fn insert_account(&mut self, address: Address, info: AccountInfo) {
        <Self as StateDb>::insert_account(self, address, info).unwrap_or_else(|e| match e {})
    }

    pub fn insert_storage(&mut self, address: Address, index: U256, value: U256) {
        <Self as StateDb>::insert_storage(self, address, index, value)
            .unwrap_or_else(|e| match e {})
    }

    pub fn insert_block_hash(&mut self, number: u64, hash: B256) {
        <Self as StateDb>::insert_block_hash(self, number, hash).unwrap_or_else(|e| match e {})
    }

    pub fn get_account(&self, address: Address) -> Option<AccountInfo> {
        <Self as StateDb>::get_account(self, address).unwrap_or_else(|e| match e {})
    }

    pub fn get_code_by_hash(&self, code_hash: B256) -> Bytecode {
        <Self as StateDb>::get_code_by_hash(self, code_hash).unwrap_or_else(|e| match e {})
    }

    pub fn get_storage(&self, address: Address, index: U256) -> U256 {
        <Self as StateDb>::get_storage(self, address, index).unwrap_or_else(|e| match e {})
    }

    pub fn get_block_hash(&self, number: u64) -> B256 {
        <Self as StateDb>::get_block_hash(self, number).unwrap_or_else(|e| match e {})
    }
}

impl DatabaseRef for InMemoryStateDb {
    type Error = StateError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(self
            .accounts
            .get(&address)
            .map(|account| account.info.clone()))
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(self.bytecodes.get(&code_hash).cloned().unwrap_or_default())
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Ok(self
            .accounts
            .get(&address)
            .and_then(|account| account.storage.get(&index).copied())
            .unwrap_or(U256::ZERO))
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        Ok(self
            .block_hashes
            .get(&number)
            .copied()
            .unwrap_or(B256::ZERO))
    }
}

impl Database for InMemoryStateDb {
    type Error = StateError;

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

#[cfg(test)]
#[path = "tests/db.rs"]
mod tests;
