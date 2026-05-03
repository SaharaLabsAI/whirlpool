use std::sync::{Arc, RwLock};
use std::thread;

use crate::RethStateDb;
use alloy_primitives::{address, U256};
use revm::state::AccountInfo;
use state::StateDb;

#[test]
fn test_concurrent_reads() {
    let mut db = RethStateDb::new();
    let addr = address!("2222222222222222222222222222222222222222");
    let info = AccountInfo {
        balance: U256::from(9_999u64),
        nonce: 2,
        code_hash: revm::primitives::KECCAK_EMPTY,
        code: None,
        account_id: None,
    };
    db.insert_account(addr, info.clone()).unwrap();

    let shared = Arc::new(db);
    let mut handles = Vec::new();

    for _ in 0..10 {
        let shared_db = Arc::clone(&shared);
        let expected = info.clone();
        handles.push(thread::spawn(move || {
            let got = shared_db.get_account(addr).unwrap();
            assert_eq!(got, Some(expected));
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_single_writer_multiple_readers() {
    let db = Arc::new(RwLock::new(RethStateDb::new()));
    let addr = address!("3333333333333333333333333333333333333333");

    {
        let mut guard = db.write().unwrap();
        guard
            .insert_account(
                addr,
                AccountInfo {
                    balance: U256::from(1u64),
                    nonce: 0,
                    code_hash: revm::primitives::KECCAK_EMPTY,
                    code: None,
                    account_id: None,
                },
            )
            .unwrap();
    }

    let writer_db = Arc::clone(&db);
    let writer = thread::spawn(move || {
        let mut guard = writer_db.write().unwrap();
        guard
            .insert_account(
                addr,
                AccountInfo {
                    balance: U256::from(2u64),
                    nonce: 1,
                    code_hash: revm::primitives::KECCAK_EMPTY,
                    code: None,
                    account_id: None,
                },
            )
            .unwrap();
    });

    let mut readers = Vec::new();
    for _ in 0..10 {
        let reader_db = Arc::clone(&db);
        readers.push(thread::spawn(move || {
            let guard = reader_db.read().unwrap();
            let got = guard.get_account(addr).unwrap();
            assert!(got.is_some());
        }));
    }

    writer.join().unwrap();
    for reader in readers {
        reader.join().unwrap();
    }

    let final_info = db.read().unwrap().get_account(addr).unwrap().unwrap();
    assert_eq!(final_info.balance, U256::from(2u64));
    assert_eq!(final_info.nonce, 1);
}
