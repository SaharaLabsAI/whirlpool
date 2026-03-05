use std::collections::{BTreeMap, HashMap};

use alloy_genesis::GenesisAccount;
use alloy_primitives::{address, b256, U256};
use state::StateDb;
use state_reth::RethStateDb;

#[test]
fn test_with_genesis_populates() {
    let addr1 = address!("4444444444444444444444444444444444444444");
    let addr2 = address!("5555555555555555555555555555555555555555");

    let mut storage = BTreeMap::new();
    storage.insert(
        b256!("0000000000000000000000000000000000000000000000000000000000000001"),
        b256!("0000000000000000000000000000000000000000000000000000000000000002"),
    );

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
                storage: Some(storage),
                ..GenesisAccount::default()
            },
        ),
    ]);

    let db = RethStateDb::with_genesis(alloc);

    let a1 = db.get_account(addr1).unwrap().unwrap();
    assert_eq!(a1.balance, U256::from(1_000_000u64));

    let slot = U256::from(1u64);
    assert_eq!(db.get_storage(addr2, slot).unwrap(), U256::from(2u64));
}

#[test]
fn test_with_genesis_root() {
    let addr = address!("6666666666666666666666666666666666666666");
    let alloc = HashMap::from([(
        addr,
        GenesisAccount {
            balance: U256::from(123u64),
            ..GenesisAccount::default()
        },
    )]);

    let db = RethStateDb::with_genesis(alloc);
    let root = db.state_root().unwrap();

    assert_ne!(root, revm::primitives::KECCAK_EMPTY);
}
