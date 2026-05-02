use alloy_primitives::{address, U256};
use app_evm_state::RethStateDb;
use revm::state::AccountInfo;
use state::StateDb;

fn seeded_db() -> RethStateDb {
    let mut db = RethStateDb::new();
    db.insert_account(
        address!("7777777777777777777777777777777777777777"),
        AccountInfo {
            balance: U256::from(10u64),
            nonce: 1,
            code_hash: revm::primitives::KECCAK_EMPTY,
            code: None,
            account_id: None,
        },
    )
    .unwrap();
    db.insert_account(
        address!("8888888888888888888888888888888888888888"),
        AccountInfo {
            balance: U256::from(20u64),
            nonce: 2,
            code_hash: revm::primitives::KECCAK_EMPTY,
            code: None,
            account_id: None,
        },
    )
    .unwrap();
    db
}

#[test]
fn test_state_root_determinism() {
    let db1 = seeded_db();
    let db2 = seeded_db();

    assert_eq!(db1.state_root().unwrap(), db2.state_root().unwrap());
}

#[test]
fn test_state_root_idempotency() {
    let db = seeded_db();

    let root1 = db.state_root().unwrap();
    let root2 = db.state_root().unwrap();

    assert_eq!(root1, root2);
}
