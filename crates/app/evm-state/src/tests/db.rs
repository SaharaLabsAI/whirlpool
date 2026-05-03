use alloy_primitives::{address, b256, Address, Bytes, U256};
use revm::database::states::StorageSlot;
use revm::database::{AccountStatus, BundleAccount, BundleState};
use revm::primitives::StorageKeyMap;
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
    let mut storage_map: StorageKeyMap<StorageSlot> = StorageKeyMap::default();
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
#[serial_test::serial]
fn test_insert_and_get_account() {
    let mut db = RethStateDb::new();
    let addr = address!("1000000000000000000000000000000000000001");
    let info = account_info(10_000, 7);

    db.insert_account(addr, info.clone()).unwrap();
    let got = db.get_account(addr).unwrap();

    assert_eq!(got, Some(info));
}

#[test]
#[serial_test::serial]
fn test_get_account_missing() {
    let db = RethStateDb::new();
    let unknown = address!("2000000000000000000000000000000000000002");

    assert_eq!(db.get_account(unknown).unwrap(), None);
}

#[test]
#[serial_test::serial]
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
#[serial_test::serial]
fn test_get_storage_missing() {
    let db = RethStateDb::new();
    let unknown = address!("4000000000000000000000000000000000000004");
    let unknown_key = U256::from(123u64);

    assert_eq!(db.get_storage(unknown, unknown_key).unwrap(), U256::ZERO);
}

#[test]
#[serial_test::serial]
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
#[serial_test::serial]
fn test_insert_and_get_block_hash() {
    let mut db = RethStateDb::new();
    let number = 42;
    let hash = b256!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    db.insert_block_hash(number, hash).unwrap();
    assert_eq!(db.get_block_hash(number).unwrap(), hash);
}

#[test]
#[serial_test::serial]
fn test_state_root_empty() {
    let db = RethStateDb::new();
    let expected = b256!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421");

    assert_eq!(db.state_root().unwrap(), expected);
}

#[test]
#[serial_test::serial]
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
#[serial_test::serial]
fn test_revm_database_basic() {
    let mut db = RethStateDb::new();
    let addr = address!("8000000000000000000000000000000000000008");
    let info = account_info(1234, 9);
    db.insert_account(addr, info.clone()).unwrap();

    let got = db.basic(addr).unwrap();
    assert_eq!(got, Some(info));
}

#[test]
#[serial_test::serial]
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
#[serial_test::serial]
fn test_revm_database_ref_basic() {
    let mut db = RethStateDb::new();
    let addr = address!("a00000000000000000000000000000000000000a");
    let info = account_info(55, 3);
    db.insert_account(addr, info.clone()).unwrap();

    let got = db.basic_ref(addr).unwrap();
    assert_eq!(got, Some(info));
}
