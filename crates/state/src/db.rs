use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use alloy_genesis::GenesisAccount;
use revm::database::BundleState;
use revm::primitives::{keccak256, Address, B256, KECCAK_EMPTY, U256};
use revm::state::{AccountInfo, Bytecode};
use revm::{Database, DatabaseRef};

use crate::error::StateError;

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

impl InMemoryStateDb {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            bytecodes: HashMap::new(),
            block_hashes: HashMap::new(),
        }
    }

    pub fn with_genesis(alloc: HashMap<Address, GenesisAccount>) -> Self {
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

    pub fn commit(&mut self, bundle: &BundleState) {
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
    }

    pub fn state_root(&self) -> B256 {
        if self.accounts.is_empty() {
            return KECCAK_EMPTY;
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

        keccak256(encoded)
    }

    pub fn insert_block_hash(&mut self, number: u64, hash: B256) {
        self.block_hashes.insert(number, hash);
    }
}

impl DatabaseRef for InMemoryStateDb {
    type Error = StateError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(self.accounts.get(&address).map(|account| account.info.clone()))
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(self
            .bytecodes
            .get(&code_hash)
            .cloned()
            .unwrap_or_default())
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Ok(self
            .accounts
            .get(&address)
            .and_then(|account| account.storage.get(&index).copied())
            .unwrap_or(U256::ZERO))
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        Ok(self.block_hashes.get(&number).copied().unwrap_or(B256::ZERO))
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
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use alloy_genesis::GenesisAccount;
    use revm::database::states::StorageSlot;
    use revm::database::{AccountStatus, BundleAccount, BundleState};
    use revm::primitives::{Address, Bytes, HashMap as RevmHashMap, B256, KECCAK_EMPTY, U256};
    use revm::state::{AccountInfo, Bytecode};
    use revm::DatabaseRef;

    use crate::db::{DbAccount, InMemoryStateDb};

    fn address(byte: u8) -> Address {
        Address::from([byte; 20])
    }

    fn b256(byte: u8) -> B256 {
        B256::from([byte; 32])
    }

    fn account_info(balance: u64, nonce: u64, code_hash: B256) -> AccountInfo {
        AccountInfo {
            balance: U256::from(balance),
            nonce,
            code_hash,
            ..AccountInfo::default()
        }
    }

    fn bundle_with_account(
        address: Address,
        original: Option<AccountInfo>,
        present: Option<AccountInfo>,
        status: AccountStatus,
        storage: &[(U256, U256, U256)],
    ) -> BundleState {
        let mut storage_map: RevmHashMap<U256, StorageSlot> = RevmHashMap::default();
        for (key, original_value, present_value) in storage {
            storage_map.insert(*key, StorageSlot::new_changed(*original_value, *present_value));
        }

        let mut bundle = BundleState::default();
        bundle.state.insert(
            address,
            BundleAccount::new(original, present, storage_map, status),
        );
        bundle
    }

    #[test]
    fn test_basic_none() {
        let db = InMemoryStateDb::new();
        assert_eq!(db.basic_ref(address(1)).unwrap(), None);
    }

    #[test]
    fn test_basic_returns_info() {
        let mut db = InMemoryStateDb::new();
        let addr = address(2);
        let info = account_info(100, 1, KECCAK_EMPTY);

        db.accounts.insert(
            addr,
            DbAccount {
                info: info.clone(),
                storage: HashMap::new(),
            },
        );

        assert_eq!(db.basic_ref(addr).unwrap(), Some(info));
    }

    #[test]
    fn test_storage_zero() {
        let db = InMemoryStateDb::new();
        assert_eq!(db.storage_ref(address(3), U256::from(7)).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_storage_value() {
        let mut db = InMemoryStateDb::new();
        let addr = address(4);

        db.accounts.insert(
            addr,
            DbAccount {
                info: AccountInfo::default(),
                storage: HashMap::from([(U256::from(1), U256::from(42))]),
            },
        );

        assert_eq!(db.storage_ref(addr, U256::from(1)).unwrap(), U256::from(42));
    }

    #[test]
    fn test_code_by_hash_default() {
        let db = InMemoryStateDb::new();
        assert_eq!(db.code_by_hash_ref(b256(1)).unwrap(), Bytecode::default());
    }

    #[test]
    fn test_block_hash_zero() {
        let db = InMemoryStateDb::new();
        assert_eq!(db.block_hash_ref(1).unwrap(), B256::ZERO);
    }

    #[test]
    fn test_block_hash_inserted() {
        let mut db = InMemoryStateDb::new();
        let hash = b256(9);
        db.insert_block_hash(11, hash);
        assert_eq!(db.block_hash_ref(11).unwrap(), hash);
    }

    #[test]
    fn test_commit_create_account() {
        let mut db = InMemoryStateDb::new();
        let addr = address(10);
        let info = account_info(1_000, 0, KECCAK_EMPTY);

        let bundle = bundle_with_account(addr, None, Some(info.clone()), AccountStatus::InMemoryChange, &[]);
        db.commit(&bundle);

        let stored = db.basic_ref(addr).unwrap().unwrap();
        assert_eq!(stored.balance, U256::from(1_000));
        assert_eq!(stored.nonce, 0);
    }

    #[test]
    fn test_commit_update_account() {
        let mut db = InMemoryStateDb::new();
        let addr = address(11);

        db.accounts.insert(
            addr,
            DbAccount {
                info: account_info(100, 0, KECCAK_EMPTY),
                storage: HashMap::new(),
            },
        );

        let updated = account_info(200, 1, KECCAK_EMPTY);
        let bundle = bundle_with_account(
            addr,
            Some(account_info(100, 0, KECCAK_EMPTY)),
            Some(updated.clone()),
            AccountStatus::Changed,
            &[],
        );
        db.commit(&bundle);

        let stored = db.basic_ref(addr).unwrap().unwrap();
        assert_eq!(stored.balance, U256::from(200));
        assert_eq!(stored.nonce, 1);
    }

    #[test]
    fn test_commit_applies_account_changes() {
        let mut db = InMemoryStateDb::new();
        let addr = address(111);

        db.accounts.insert(
            addr,
            DbAccount {
                info: account_info(1_000, 2, KECCAK_EMPTY),
                storage: HashMap::new(),
            },
        );

        let updated = account_info(3_000, 7, KECCAK_EMPTY);
        let bundle = bundle_with_account(
            addr,
            Some(account_info(1_000, 2, KECCAK_EMPTY)),
            Some(updated.clone()),
            AccountStatus::Changed,
            &[],
        );
        db.commit(&bundle);

        let stored = db.basic_ref(addr).unwrap().unwrap();
        assert_eq!(stored.balance, U256::from(3_000));
        assert_eq!(stored.nonce, 7);
    }

    #[test]
    fn test_commit_destroy_account() {
        let mut db = InMemoryStateDb::new();
        let addr = address(12);

        db.accounts.insert(
            addr,
            DbAccount {
                info: account_info(100, 0, KECCAK_EMPTY),
                storage: HashMap::from([(U256::from(1), U256::from(2))]),
            },
        );

        let bundle = bundle_with_account(
            addr,
            Some(account_info(100, 0, KECCAK_EMPTY)),
            None,
            AccountStatus::Destroyed,
            &[],
        );
        db.commit(&bundle);

        assert_eq!(db.basic_ref(addr).unwrap(), None);
    }

    #[test]
    fn test_commit_storage_changes() {
        let mut db = InMemoryStateDb::new();
        let addr = address(13);

        db.accounts.insert(
            addr,
            DbAccount {
                info: account_info(1, 0, KECCAK_EMPTY),
                storage: HashMap::from([(U256::from(1), U256::from(5))]),
            },
        );

        let bundle = bundle_with_account(
            addr,
            Some(account_info(1, 0, KECCAK_EMPTY)),
            Some(account_info(1, 0, KECCAK_EMPTY)),
            AccountStatus::Changed,
            &[
                (U256::from(1), U256::from(5), U256::ZERO),
                (U256::from(2), U256::ZERO, U256::from(99)),
            ],
        );
        db.commit(&bundle);

        assert_eq!(db.storage_ref(addr, U256::from(1)).unwrap(), U256::ZERO);
        assert_eq!(db.storage_ref(addr, U256::from(2)).unwrap(), U256::from(99));
    }

    #[test]
    fn test_commit_new_bytecode() {
        let mut db = InMemoryStateDb::new();
        let code = Bytecode::new_raw(Bytes::from(vec![0x60, 0x00]));
        let hash = code.hash_slow();

        let mut bundle = BundleState::default();
        bundle.contracts.insert(hash, code.clone());

        db.commit(&bundle);
        assert_eq!(db.code_by_hash_ref(hash).unwrap(), code);
    }

    #[test]
    fn test_state_root_deterministic() {
        let mut db1 = InMemoryStateDb::new();
        let mut db2 = InMemoryStateDb::new();
        let addr = address(14);
        let info = account_info(100, 1, KECCAK_EMPTY);

        db1.accounts.insert(
            addr,
            DbAccount {
                info: info.clone(),
                storage: HashMap::from([(U256::from(1), U256::from(10))]),
            },
        );
        db2.accounts.insert(
            addr,
            DbAccount {
                info,
                storage: HashMap::from([(U256::from(1), U256::from(10))]),
            },
        );

        assert_eq!(db1.state_root(), db2.state_root());
    }

    #[test]
    fn test_state_root_changes_after_commit() {
        let mut db = InMemoryStateDb::new();
        let root_before = db.state_root();

        let bundle = bundle_with_account(
            address(15),
            None,
            Some(account_info(1, 0, KECCAK_EMPTY)),
            AccountStatus::InMemoryChange,
            &[],
        );
        db.commit(&bundle);

        assert_ne!(db.state_root(), root_before);
    }

    #[test]
    fn test_state_root_empty_db() {
        let db = InMemoryStateDb::new();
        assert_eq!(db.state_root(), KECCAK_EMPTY);
    }

    #[test]
    fn test_independent_snapshot() {
        let mut db = InMemoryStateDb::new();
        let addr = address(16);

        db.accounts.insert(
            addr,
            DbAccount {
                info: account_info(100, 0, KECCAK_EMPTY),
                storage: HashMap::new(),
            },
        );

        let mut clone = db.clone();
        let bundle = bundle_with_account(
            addr,
            Some(account_info(100, 0, KECCAK_EMPTY)),
            Some(account_info(200, 1, KECCAK_EMPTY)),
            AccountStatus::Changed,
            &[],
        );
        clone.commit(&bundle);

        assert_eq!(db.basic_ref(addr).unwrap().unwrap().balance, U256::from(100));
        assert_eq!(clone.basic_ref(addr).unwrap().unwrap().balance, U256::from(200));
    }

    #[test]
    fn test_with_genesis_populates() {
        let addr1 = address(21);
        let addr2 = address(22);

        let mut storage = BTreeMap::new();
        storage.insert(b256(1), b256(2));

        let alloc = HashMap::from([
            (
                addr1,
                GenesisAccount {
                    balance: U256::from(1_000_000u64),
                    ..GenesisAccount::default()
                },
            ),
            (
                addr2,
                GenesisAccount {
                    code: Some(Bytes::from(vec![0x60, 0x00])),
                    storage: Some(storage),
                    ..GenesisAccount::default()
                },
            ),
        ]);

        let db = InMemoryStateDb::with_genesis(alloc);

        assert_eq!(
            db.basic_ref(addr1).unwrap().unwrap().balance,
            U256::from(1_000_000u64)
        );
        let code_hash = db.basic_ref(addr2).unwrap().unwrap().code_hash;
        assert_eq!(
            db.code_by_hash_ref(code_hash).unwrap(),
            Bytecode::new_raw(Bytes::from(vec![0x60, 0x00]))
        );
        assert_eq!(
            db.storage_ref(addr2, U256::from_be_bytes(b256(1).0)).unwrap(),
            U256::from_be_bytes(b256(2).0)
        );
    }

    #[test]
    fn test_state_root_account_ordering() {
        let mut db1 = InMemoryStateDb::new();
        let mut db2 = InMemoryStateDb::new();

        let a1 = address(31);
        let a2 = address(32);

        db1.accounts.insert(
            a1,
            DbAccount {
                info: account_info(10, 1, KECCAK_EMPTY),
                storage: HashMap::from([(U256::from(3), U256::from(7))]),
            },
        );
        db1.accounts.insert(
            a2,
            DbAccount {
                info: account_info(11, 2, KECCAK_EMPTY),
                storage: HashMap::from([(U256::from(4), U256::from(8))]),
            },
        );

        db2.accounts.insert(
            a2,
            DbAccount {
                info: account_info(11, 2, KECCAK_EMPTY),
                storage: HashMap::from([(U256::from(4), U256::from(8))]),
            },
        );
        db2.accounts.insert(
            a1,
            DbAccount {
                info: account_info(10, 1, KECCAK_EMPTY),
                storage: HashMap::from([(U256::from(3), U256::from(7))]),
            },
        );

        assert_eq!(db1.state_root(), db2.state_root());
    }
}
