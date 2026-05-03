// Core database module implementing RethStateDb — persistent state backed by MDBX.
//
// RethStateDb wraps a reth `DatabaseEnv` and implements:
// - `state::StateDb` for the whirlpool state interface
// - `revm::Database` / `revm::DatabaseRef` for EVM execution
//
// Each method opens a short-lived MDBX transaction. The caller is responsible
// for synchronization (typically via `Arc<RwLock<RethStateDb>>`).

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

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
use state::StateDb;

use crate::codec::{account_to_info, info_to_account};
use crate::error::RethStateError;
use crate::reth::rpc_reader::RpcStateReader;
use crate::reth::trie::compute_state_root;
use reth_db_api::tables::{
    Bytecodes, CanonicalHeaders, HashedAccounts, HashedStorages, PlainAccountState,
    PlainStorageState,
};

// Shared temp directories kept alive for DBs created via `StateDb::new`.
static TEST_DB_TEMP_DIRS: OnceLock<Mutex<Vec<Arc<tempfile::TempDir>>>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct RethStateDb {
    pub(in crate::reth) db: Arc<DatabaseEnv>,
}

fn genesis_account_info(
    tx: &impl DbTxMut,
    account: &GenesisAccount,
) -> Result<AccountInfo, RethStateError> {
    let mut info = AccountInfo {
        balance: account.balance,
        nonce: account.nonce.unwrap_or_default(),
        code_hash: KECCAK_EMPTY,
        code: None,
        account_id: None,
    };

    let Some(code_bytes) = &account.code else {
        return Ok(info);
    };

    let code = Bytecode::new_raw(code_bytes.clone());
    let code_hash = code.hash_slow();
    info.code_hash = code_hash;
    tx.put::<Bytecodes>(code_hash, reth_primitives_traits::Bytecode(code))
        .map_err(RethStateError::Database)?;
    Ok(info)
}

fn write_account_state(
    tx: &impl DbTxMut,
    address: Address,
    info: &AccountInfo,
) -> Result<B256, RethStateError> {
    let reth_account = info_to_account(info);
    tx.put::<PlainAccountState>(address, reth_account)
        .map_err(RethStateError::Database)?;
    let hashed_addr = keccak256(address);
    tx.put::<HashedAccounts>(hashed_addr, reth_account)
        .map_err(RethStateError::Database)?;
    Ok(hashed_addr)
}

fn write_genesis_storage(
    tx: &impl DbTxMut,
    address: Address,
    hashed_addr: B256,
    genesis_storage: &BTreeMap<B256, B256>,
) -> Result<(), RethStateError> {
    for (key, value) in genesis_storage {
        let slot = U256::from_be_bytes(key.0);
        let val = U256::from_be_bytes(value.0);
        if val.is_zero() {
            continue;
        }

        let key_b256 = B256::from(slot.to_be_bytes::<32>());
        let entry = StorageEntry::new(key_b256, val);
        let mut cursor = tx
            .cursor_dup_write::<PlainStorageState>()
            .map_err(RethStateError::Database)?;
        cursor
            .upsert(address, &entry)
            .map_err(RethStateError::Database)?;

        let hashed_entry = StorageEntry::new(keccak256(key_b256), val);
        let mut hcursor = tx
            .cursor_dup_write::<HashedStorages>()
            .map_err(RethStateError::Database)?;
        hcursor
            .upsert(hashed_addr, &hashed_entry)
            .map_err(RethStateError::Database)?;
    }
    Ok(())
}

fn write_genesis_account(
    tx: &impl DbTxMut,
    address: Address,
    account: &GenesisAccount,
) -> Result<(), RethStateError> {
    let info = genesis_account_info(tx, account)?;
    let hashed_addr = write_account_state(tx, address, &info)?;
    let Some(genesis_storage) = &account.storage else {
        return Ok(());
    };
    write_genesis_storage(tx, address, hashed_addr, genesis_storage)
}

fn delete_account_state(
    tx: &impl DbTxMut,
    address: Address,
    hashed_addr: B256,
) -> Result<(), RethStateError> {
    tx.delete::<PlainAccountState>(address, None)
        .map_err(RethStateError::Database)?;
    tx.delete::<HashedAccounts>(hashed_addr, None)
        .map_err(RethStateError::Database)?;
    tx.delete::<PlainStorageState>(address, None)
        .map_err(RethStateError::Database)?;
    tx.delete::<HashedStorages>(hashed_addr, None)
        .map_err(RethStateError::Database)?;
    Ok(())
}

fn delete_storage_slot(
    tx: &impl DbTxMut,
    address: Address,
    hashed_addr: B256,
    key_b256: B256,
    hashed_slot: B256,
) -> Result<(), RethStateError> {
    tx.delete::<PlainStorageState>(address, Some(StorageEntry::new(key_b256, U256::ZERO)))
        .map_err(RethStateError::Database)?;
    tx.delete::<HashedStorages>(
        hashed_addr,
        Some(StorageEntry::new(hashed_slot, U256::ZERO)),
    )
    .map_err(RethStateError::Database)?;
    Ok(())
}

fn upsert_storage_slot(
    tx: &impl DbTxMut,
    address: Address,
    hashed_addr: B256,
    key_b256: B256,
    hashed_slot: B256,
    value: U256,
) -> Result<(), RethStateError> {
    let mut cursor = tx
        .cursor_dup_write::<PlainStorageState>()
        .map_err(RethStateError::Database)?;
    if cursor
        .seek_by_key_subkey(address, key_b256)
        .map_err(RethStateError::Database)?
        .is_some()
    {
        cursor.delete_current().map_err(RethStateError::Database)?;
    }
    cursor
        .upsert(address, &StorageEntry::new(key_b256, value))
        .map_err(RethStateError::Database)?;

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
        .upsert(hashed_addr, &StorageEntry::new(hashed_slot, value))
        .map_err(RethStateError::Database)?;
    Ok(())
}

fn write_bundle_storage_slot(
    tx: &impl DbTxMut,
    address: Address,
    hashed_addr: B256,
    key: U256,
    value: U256,
) -> Result<(), RethStateError> {
    let key_b256 = B256::from(key.to_be_bytes::<32>());
    let hashed_slot = keccak256(key_b256);
    if value.is_zero() {
        return delete_storage_slot(tx, address, hashed_addr, key_b256, hashed_slot);
    }
    upsert_storage_slot(tx, address, hashed_addr, key_b256, hashed_slot, value)
}

impl RethStateDb {
    /// Open (or create) the MDBX database at the given path.
    pub fn open(path: &Path) -> Result<Self, RethStateError> {
        let db = init_db(path, DatabaseArguments::new(ClientVersion::default()))
            .map_err(|e| RethStateError::Init(e.to_string()))?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Apply genesis account allocations to the database.
    ///
    /// Writes account balances, nonces, bytecode, and storage slots from the
    /// genesis alloc to the state tables. This should be called once after
    /// opening a fresh database before starting consensus.
    pub fn apply_genesis(
        &self,
        alloc: &HashMap<Address, GenesisAccount>,
    ) -> Result<(), RethStateError> {
        let tx = self.db.tx_mut().map_err(RethStateError::Database)?;
        for (address, account) in alloc {
            write_genesis_account(&tx, *address, account)?;
        }
        tx.commit().map_err(RethStateError::Database)?;
        Ok(())
    }

    pub fn rpc_reader(&self) -> RpcStateReader<'_> {
        RpcStateReader { db: self }
    }
}

impl StateDb for RethStateDb {
    type Error = RethStateError;

    fn new() -> Self
    where
        Self: Sized,
    {
        // RethStateDb requires a path — keep temp dirs alive globally in tests.
        let temp_dir =
            Arc::new(tempfile::tempdir().expect("failed to create tempdir for RethStateDb::new()"));
        let db = init_db(
            temp_dir.path(),
            DatabaseArguments::new(ClientVersion::default()),
        )
        .expect("failed to initialize RethStateDb database");

        TEST_DB_TEMP_DIRS
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .expect("failed to lock test db tempdir registry")
            .push(temp_dir);

        Self { db: Arc::new(db) }
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
                write_genesis_account(&tx, *address, account).expect("failed to write genesis");
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
                delete_account_state(&tx, *address, hashed_addr)?;
                continue;
            }

            let Some(info) = bundle_account.account_info() else {
                delete_account_state(&tx, *address, hashed_addr)?;
                continue;
            };

            write_account_state(&tx, *address, &info)?;

            // Handle storage changes.
            if bundle_account.status.is_storage_known() {
                // Wipe existing storage.
                tx.delete::<PlainStorageState>(*address, None)
                    .map_err(RethStateError::Database)?;
                tx.delete::<HashedStorages>(hashed_addr, None)
                    .map_err(RethStateError::Database)?;
            }

            for (key, slot) in &bundle_account.storage {
                let value = slot.present_value();
                write_bundle_storage_slot(&tx, *address, hashed_addr, *key, value)?;
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

    fn insert_storage(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Result<(), Self::Error> {
        let tx = self.db.tx_mut().map_err(RethStateError::Database)?;
        let key = B256::from(index.to_be_bytes::<32>());
        let hashed_addr = keccak256(address);
        let hashed_slot = keccak256(key);

        // Ensure account exists so storage is tied to a canonical state account.
        if tx
            .get::<PlainAccountState>(address)
            .map_err(RethStateError::Database)?
            .is_none()
        {
            let empty = info_to_account(&AccountInfo::default());
            tx.put::<PlainAccountState>(address, empty)
                .map_err(RethStateError::Database)?;
            tx.put::<HashedAccounts>(hashed_addr, empty)
                .map_err(RethStateError::Database)?;
        }

        if value.is_zero() {
            let plain_entry = StorageEntry::new(key, U256::ZERO);
            tx.delete::<PlainStorageState>(address, Some(plain_entry))
                .map_err(RethStateError::Database)?;

            let hashed_entry = StorageEntry::new(hashed_slot, U256::ZERO);
            tx.delete::<HashedStorages>(hashed_addr, Some(hashed_entry))
                .map_err(RethStateError::Database)?;
        } else {
            let plain_entry = StorageEntry::new(key, value);
            let mut cursor = tx
                .cursor_dup_write::<PlainStorageState>()
                .map_err(RethStateError::Database)?;
            cursor
                .upsert(address, &plain_entry)
                .map_err(RethStateError::Database)?;

            let hashed_entry = StorageEntry::new(hashed_slot, value);
            let mut hcursor = tx
                .cursor_dup_write::<HashedStorages>()
                .map_err(RethStateError::Database)?;
            hcursor
                .upsert(hashed_addr, &hashed_entry)
                .map_err(RethStateError::Database)?;
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
