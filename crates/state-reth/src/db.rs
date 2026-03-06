// Core database module implementing RethStateDb — persistent state backed by MDBX.
//
// RethStateDb wraps a reth `DatabaseEnv` and implements:
// - `state::StateDb` for the whirlpool state interface
// - `revm::Database` / `revm::DatabaseRef` for EVM execution
//
// Each method opens a short-lived MDBX transaction. The caller is responsible
// for synchronization (typically via `Arc<RwLock<RethStateDb>>`).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use alloy_genesis::GenesisAccount;
use alloy_primitives::{keccak256, B256, U256};
use reth_db::mdbx::DatabaseArguments;
use reth_db::{init_db, ClientVersion, Database, DatabaseEnv};
use reth_db_api::cursor::{DbCursorRW, DbDupCursorRO};
use reth_db_api::transaction::{DbTx, DbTxMut};
use reth_primitives_traits::StorageEntry;
use revm::database::BundleState;
use revm::primitives::{Address, KECCAK_EMPTY};
use revm::state::{AccountInfo, Bytecode};
use revm::DatabaseRef;
use state::StateDb;

use crate::codec::{account_to_info, info_to_account};
use crate::error::RethStateError;
use crate::tables::{
    Bytecodes, CanonicalHeaders, HashedAccounts, HashedStorages, PlainAccountState,
    PlainStorageState,
};
use crate::trie::compute_state_root;

/// Persistent state database backed by reth-db (MDBX).
///
/// All public methods open per-call read or write transactions.
/// The struct itself is `Clone` (via `Arc<DatabaseEnv>`).
#[derive(Debug, Clone)]
pub struct RethStateDb {
    db: Arc<DatabaseEnv>,
}

impl RethStateDb {
    /// Open (or create) the MDBX database at the given path.
    pub fn open(path: &Path) -> Result<Self, RethStateError> {
        let db = init_db(path, DatabaseArguments::new(ClientVersion::default()))
            .map_err(|e| RethStateError::Init(e.to_string()))?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Access the underlying `DatabaseEnv` for advanced operations.
    pub fn inner(&self) -> &DatabaseEnv {
        &self.db
    }
}

impl StateDb for RethStateDb {
    type Error = RethStateError;

    fn new() -> Self
    where
        Self: Sized,
    {
        // RethStateDb requires a path — use a temp dir for `new()`.
        // This is primarily for trait compliance; production code should use `open()`.
        let tmp = tempfile::tempdir().expect("failed to create tempdir for RethStateDb::new()");
        Self::open(tmp.path()).expect("failed to initialize RethStateDb")
    }

    fn with_genesis(alloc: HashMap<Address, GenesisAccount>) -> Self
    where
        Self: Sized,
    {
        let db = Self::new();
        // Write genesis allocations. Re-use the StateDb::insert_account and
        // direct storage/bytecode writes.
        {
            let tx = db.db.tx_mut().expect("failed to open write tx for genesis");
            for (address, account) in &alloc {
                let nonce = account.nonce.unwrap_or_default();
                let mut info = AccountInfo {
                    balance: account.balance,
                    nonce,
                    code_hash: KECCAK_EMPTY,
                    code: None,
                    account_id: None,
                };

                // Store bytecode if present.
                if let Some(ref code_bytes) = account.code {
                    let code = Bytecode::new_raw(code_bytes.clone());
                    let code_hash = code.hash_slow();
                    info.code_hash = code_hash;
                    tx.put::<Bytecodes>(code_hash, reth_primitives_traits::Bytecode(code))
                        .expect("failed to write bytecode");
                }

                // Store account in plain + hashed tables.
                let reth_account = info_to_account(&info);
                tx.put::<PlainAccountState>(*address, reth_account)
                    .expect("failed to write plain account");
                let hashed_addr = keccak256(address);
                tx.put::<HashedAccounts>(hashed_addr, reth_account)
                    .expect("failed to write hashed account");

                // Store genesis storage if present.
                if let Some(ref genesis_storage) = account.storage {
                    for (key, value) in genesis_storage {
                        let slot = U256::from_be_bytes(key.0);
                        let val = U256::from_be_bytes(value.0);
                        if !val.is_zero() {
                            let entry =
                                StorageEntry::new(B256::from(slot.to_be_bytes::<32>()), val);
                            // Plain storage
                            let mut cursor =
                                tx.cursor_dup_write::<PlainStorageState>().expect("cursor");
                            cursor
                                .upsert(*address, &entry)
                                .expect("plain storage write");
                            // Hashed storage
                            let hashed_slot = keccak256(B256::from(slot.to_be_bytes::<32>()));
                            let hashed_entry = StorageEntry::new(hashed_slot, val);
                            let mut hcursor =
                                tx.cursor_dup_write::<HashedStorages>().expect("cursor");
                            hcursor
                                .upsert(hashed_addr, &hashed_entry)
                                .expect("hashed storage write");
                        }
                    }
                }
            }
            tx.commit().expect("failed to commit genesis");
        }
        db
    }

    fn state_root(&self) -> Result<B256, Self::Error> {
        let tx = self.db.tx().map_err(RethStateError::Database)?;
        compute_state_root(&tx)
    }

    fn commit(&mut self, bundle: &BundleState) -> Result<(), Self::Error> {
        let tx = self.db.tx_mut().map_err(RethStateError::Database)?;

        for (address, bundle_account) in &bundle.state {
            let hashed_addr = keccak256(address);

            if bundle_account.was_destroyed() {
                // Delete account from plain + hashed tables.
                let _ = tx.delete::<PlainAccountState>(*address, None);
                let _ = tx.delete::<HashedAccounts>(hashed_addr, None);
                // Delete all storage for this account.
                let _ = tx.delete::<PlainStorageState>(*address, None);
                let _ = tx.delete::<HashedStorages>(hashed_addr, None);
                continue;
            }

            let Some(info) = bundle_account.account_info() else {
                // Account no longer exists — remove.
                let _ = tx.delete::<PlainAccountState>(*address, None);
                let _ = tx.delete::<HashedAccounts>(hashed_addr, None);
                let _ = tx.delete::<PlainStorageState>(*address, None);
                let _ = tx.delete::<HashedStorages>(hashed_addr, None);
                continue;
            };

            // Upsert account in plain + hashed tables.
            let reth_account = info_to_account(&info);
            tx.put::<PlainAccountState>(*address, reth_account)
                .map_err(RethStateError::Database)?;
            tx.put::<HashedAccounts>(hashed_addr, reth_account)
                .map_err(RethStateError::Database)?;

            // Handle storage changes.
            if bundle_account.status.is_storage_known() {
                // Wipe existing storage.
                let _ = tx.delete::<PlainStorageState>(*address, None);
                let _ = tx.delete::<HashedStorages>(hashed_addr, None);
            }

            for (key, slot) in &bundle_account.storage {
                let value = slot.present_value();
                let key_b256 = B256::from(key.to_be_bytes::<32>());
                let hashed_slot = keccak256(key_b256);

                if value.is_zero() {
                    // Delete this specific storage slot.
                    let entry = StorageEntry::new(key_b256, U256::ZERO);
                    let _ = tx.delete::<PlainStorageState>(*address, Some(entry));
                    let hashed_entry = StorageEntry::new(hashed_slot, U256::ZERO);
                    let _ = tx.delete::<HashedStorages>(hashed_addr, Some(hashed_entry));
                } else {
                    // Upsert storage slot.
                    let entry = StorageEntry::new(key_b256, value);
                    let mut cursor = tx
                        .cursor_dup_write::<PlainStorageState>()
                        .map_err(RethStateError::Database)?;
                    // Delete old entry first, then insert new.
                    if cursor
                        .seek_by_key_subkey(*address, key_b256)
                        .map_err(RethStateError::Database)?
                        .is_some()
                    {
                        cursor.delete_current().map_err(RethStateError::Database)?;
                    }
                    cursor
                        .upsert(*address, &entry)
                        .map_err(RethStateError::Database)?;

                    let hashed_entry = StorageEntry::new(hashed_slot, value);
                    let mut hcursor = tx
                        .cursor_dup_write::<HashedStorages>()
                        .map_err(RethStateError::Database)?;
                    if hcursor
                        .seek_by_key_subkey(hashed_addr, hashed_slot)
                        .map_err(RethStateError::Database)?
                        .is_some()
                    {
                        hcursor.delete_current().map_err(RethStateError::Database)?;
                    }
                    hcursor
                        .upsert(hashed_addr, &hashed_entry)
                        .map_err(RethStateError::Database)?;
                }
            }
        }

        // Store new bytecodes.
        for (code_hash, bytecode) in &bundle.contracts {
            tx.put::<Bytecodes>(
                *code_hash,
                reth_primitives_traits::Bytecode(bytecode.clone()),
            )
            .map_err(RethStateError::Database)?;
        }

        tx.commit().map_err(RethStateError::Database)?;
        Ok(())
    }

    fn get_account(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let tx = self.db.tx().map_err(RethStateError::Database)?;
        match tx
            .get::<PlainAccountState>(address)
            .map_err(RethStateError::Database)?
        {
            Some(account) => Ok(Some(account_to_info(&account))),
            None => Ok(None),
        }
    }

    fn get_code_by_hash(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        if code_hash == KECCAK_EMPTY {
            return Ok(Bytecode::default());
        }
        let tx = self.db.tx().map_err(RethStateError::Database)?;
        Ok(tx
            .get::<Bytecodes>(code_hash)
            .map_err(RethStateError::Database)?
            .map(|b| b.0)
            .unwrap_or_default())
    }

    fn get_storage(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        let tx = self.db.tx().map_err(RethStateError::Database)?;
        let key = B256::from(index.to_be_bytes::<32>());
        let mut cursor = tx
            .cursor_dup_read::<PlainStorageState>()
            .map_err(RethStateError::Database)?;
        match cursor
            .seek_by_key_subkey(address, key)
            .map_err(RethStateError::Database)?
        {
            Some(entry) if entry.key == key => Ok(entry.value),
            _ => Ok(U256::ZERO),
        }
    }

    fn get_block_hash(&self, number: u64) -> Result<B256, Self::Error> {
        let tx = self.db.tx().map_err(RethStateError::Database)?;
        Ok(tx
            .get::<CanonicalHeaders>(number)
            .map_err(RethStateError::Database)?
            .unwrap_or(B256::ZERO))
    }

    fn insert_account(&mut self, address: Address, info: AccountInfo) -> Result<(), Self::Error> {
        let tx = self.db.tx_mut().map_err(RethStateError::Database)?;
        let reth_account = info_to_account(&info);
        tx.put::<PlainAccountState>(address, reth_account)
            .map_err(RethStateError::Database)?;
        let hashed_addr = keccak256(address);
        tx.put::<HashedAccounts>(hashed_addr, reth_account)
            .map_err(RethStateError::Database)?;

        // Store bytecode if present.
        if let Some(ref code) = info.code {
            if info.code_hash != KECCAK_EMPTY {
                tx.put::<Bytecodes>(
                    info.code_hash,
                    reth_primitives_traits::Bytecode(code.clone()),
                )
                .map_err(RethStateError::Database)?;
            }
        }

        tx.commit().map_err(RethStateError::Database)?;
        Ok(())
    }

    fn insert_block_hash(&mut self, number: u64, hash: B256) -> Result<(), Self::Error> {
        let tx = self.db.tx_mut().map_err(RethStateError::Database)?;
        tx.put::<CanonicalHeaders>(number, hash)
            .map_err(RethStateError::Database)?;
        tx.commit().map_err(RethStateError::Database)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// revm DatabaseRef impl
// ---------------------------------------------------------------------------

impl revm::DatabaseRef for RethStateDb {
    type Error = state::StateError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        StateDb::get_account(self, address).map_err(|e| state::StateError::Internal(e.to_string()))
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        StateDb::get_code_by_hash(self, code_hash)
            .map_err(|e| state::StateError::Internal(e.to_string()))
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        StateDb::get_storage(self, address, index)
            .map_err(|e| state::StateError::Internal(e.to_string()))
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        StateDb::get_block_hash(self, number)
            .map_err(|e| state::StateError::Internal(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// revm Database impl (delegates to DatabaseRef via WrapDatabaseRef)
// ---------------------------------------------------------------------------

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

#[cfg(test)]
mod tests {
    use alloy_primitives::{address, b256, Address, Bytes, U256};
    use revm::database::states::StorageSlot;
    use revm::database::{AccountStatus, BundleAccount, BundleState};
    use revm::primitives::HashMap as RevmHashMap;
    use revm::state::{AccountInfo, Bytecode};
    use revm::{Database, DatabaseRef};
    use state::StateDb;

    use crate::db::RethStateDb;

    fn account_info(balance: u64, nonce: u64) -> AccountInfo {
        AccountInfo {
            balance: U256::from(balance),
            nonce,
            code_hash: revm::primitives::KECCAK_EMPTY,
            code: None,
            account_id: None,
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
            storage_map.insert(
                *key,
                StorageSlot::new_changed(*original_value, *present_value),
            );
        }

        let mut bundle = BundleState::default();
        bundle.state.insert(
            address,
            BundleAccount::new(original, present, storage_map, status),
        );
        bundle
    }

    #[test]
    fn test_insert_and_get_account() {
        let mut db = RethStateDb::new();
        let addr = address!("1000000000000000000000000000000000000001");
        let info = account_info(10_000, 7);

        db.insert_account(addr, info.clone()).unwrap();
        let got = db.get_account(addr).unwrap();

        assert_eq!(got, Some(info));
    }

    #[test]
    fn test_get_account_missing() {
        let db = RethStateDb::new();
        let unknown = address!("2000000000000000000000000000000000000002");

        assert_eq!(db.get_account(unknown).unwrap(), None);
    }

    #[test]
    fn test_commit_storage_and_get() {
        let mut db = RethStateDb::new();
        let addr = address!("3000000000000000000000000000000000000003");
        let key = U256::from(5u64);
        let value = U256::from(77u64);

        let bundle = bundle_with_account(
            addr,
            None,
            Some(account_info(1_000, 1)),
            AccountStatus::InMemoryChange,
            &[(key, U256::ZERO, value)],
        );

        db.commit(&bundle).unwrap();
        assert_eq!(db.get_storage(addr, key).unwrap(), value);
    }

    #[test]
    fn test_get_storage_missing() {
        let db = RethStateDb::new();
        let unknown = address!("4000000000000000000000000000000000000004");
        let unknown_key = U256::from(123u64);

        assert_eq!(db.get_storage(unknown, unknown_key).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_commit_code_and_get() {
        let mut db = RethStateDb::new();
        let addr = address!("5000000000000000000000000000000000000005");
        let code = Bytecode::new_raw(Bytes::from(vec![0x60, 0x01, 0x60, 0x00]));
        let code_hash = code.hash_slow();

        let mut info = account_info(999, 2);
        info.code_hash = code_hash;
        info.code = Some(code.clone());

        let mut bundle =
            bundle_with_account(addr, None, Some(info), AccountStatus::InMemoryChange, &[]);
        bundle.contracts.insert(code_hash, code.clone());

        db.commit(&bundle).unwrap();
        assert_eq!(db.get_code_by_hash(code_hash).unwrap(), code);
    }

    #[test]
    fn test_insert_and_get_block_hash() {
        let mut db = RethStateDb::new();
        let number = 42;
        let hash = b256!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

        db.insert_block_hash(number, hash).unwrap();
        assert_eq!(db.get_block_hash(number).unwrap(), hash);
    }

    #[test]
    fn test_state_root_empty() {
        let db = RethStateDb::new();
        let expected = b256!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421");

        assert_eq!(db.state_root().unwrap(), expected);
    }

    #[test]
    fn test_state_root_deterministic() {
        let mut db = RethStateDb::new();
        db.insert_account(
            address!("6000000000000000000000000000000000000006"),
            account_info(1, 1),
        )
        .unwrap();
        db.insert_account(
            address!("7000000000000000000000000000000000000007"),
            account_info(2, 2),
        )
        .unwrap();

        let root1 = db.state_root().unwrap();
        let root2 = db.state_root().unwrap();

        assert_eq!(root1, root2);
    }

    #[test]
    fn test_revm_database_basic() {
        let mut db = RethStateDb::new();
        let addr = address!("8000000000000000000000000000000000000008");
        let info = account_info(1234, 9);
        db.insert_account(addr, info.clone()).unwrap();

        let got = db.basic(addr).unwrap();
        assert_eq!(got, Some(info));
    }

    #[test]
    fn test_revm_database_storage() {
        let mut db = RethStateDb::new();
        let addr = address!("9000000000000000000000000000000000000009");
        let key = U256::from(10u64);
        let value = U256::from(20u64);
        let bundle = bundle_with_account(
            addr,
            None,
            Some(account_info(1, 0)),
            AccountStatus::InMemoryChange,
            &[(key, U256::ZERO, value)],
        );

        db.commit(&bundle).unwrap();
        assert_eq!(db.storage(addr, key).unwrap(), value);
    }

    #[test]
    fn test_revm_database_ref_basic() {
        let mut db = RethStateDb::new();
        let addr = address!("a00000000000000000000000000000000000000a");
        let info = account_info(55, 3);
        db.insert_account(addr, info.clone()).unwrap();

        let got = db.basic_ref(addr).unwrap();
        assert_eq!(got, Some(info));
    }
}
