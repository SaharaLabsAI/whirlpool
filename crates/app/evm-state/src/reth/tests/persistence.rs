use crate::RethStateDb;
use alloy_primitives::{address, U256};
use revm::state::AccountInfo;
use state::StateDb;
use tempfile::tempdir;

#[test]
fn test_commit_durability() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    let addr = address!("1111111111111111111111111111111111111111");
    let info = AccountInfo {
        balance: U256::from(123_456u64),
        nonce: 7,
        code_hash: revm::primitives::KECCAK_EMPTY,
        code: None,
        account_id: None,
    };

    {
        let mut db = RethStateDb::open(path).unwrap();
        db.insert_account(addr, info.clone()).unwrap();
    }

    let db = RethStateDb::open(path).unwrap();
    let got = db.get_account(addr).unwrap();
    assert_eq!(got, Some(info));
}
