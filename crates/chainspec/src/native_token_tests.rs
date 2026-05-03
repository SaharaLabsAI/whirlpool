use alloy_genesis::GenesisAccount;
use alloy_primitives::{Address, U256};
use evm_precompiles::{epoch_system_tx_sender, EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI};
use std::collections::BTreeMap;

use crate::native_token::{
    sahara_hard_cap_base_units, total_allocated_supply, validate_genesis_alloc, NativeTokenError,
    SAHARA_DECIMALS, SAHARA_HARD_CAP_TOKENS,
};

#[test]
fn hard_cap_matches_expected_eth_like_base_units() {
    assert_eq!(SAHARA_DECIMALS, 18);
    assert_eq!(SAHARA_HARD_CAP_TOKENS, 10_000_000_000);
    assert_eq!(
        sahara_hard_cap_base_units(),
        U256::from(10_000_000_000_000_000_000_000_000_000u128)
    );
}

#[test]
fn validate_accepts_exact_cap() {
    let mut alloc = BTreeMap::new();
    alloc.insert(
        Address::repeat_byte(0x11),
        GenesisAccount {
            balance: sahara_hard_cap_base_units(),
            ..GenesisAccount::default()
        },
    );

    assert_eq!(
        validate_genesis_alloc(&alloc),
        Ok(sahara_hard_cap_base_units())
    );
}

#[test]
fn validate_rejects_over_cap() {
    let mut alloc = BTreeMap::new();
    let total = sahara_hard_cap_base_units() + U256::from(1u64);
    alloc.insert(
        Address::repeat_byte(0x22),
        GenesisAccount {
            balance: total,
            ..GenesisAccount::default()
        },
    );

    assert_eq!(
        validate_genesis_alloc(&alloc),
        Err(NativeTokenError::HardCapExceeded {
            total,
            hard_cap: sahara_hard_cap_base_units(),
        })
    );
}

#[test]
fn total_supply_sums_multiple_accounts() {
    let mut alloc = BTreeMap::new();
    alloc.insert(
        Address::repeat_byte(0x33),
        GenesisAccount {
            balance: U256::from(7u64),
            ..GenesisAccount::default()
        },
    );
    alloc.insert(
        Address::repeat_byte(0x44),
        GenesisAccount {
            balance: U256::from(9u64),
            ..GenesisAccount::default()
        },
    );

    assert_eq!(total_allocated_supply(&alloc), Ok(U256::from(16u64)));
}

#[test]
fn validate_ignores_epoch_system_sender_balance_for_hard_cap() {
    let mut alloc = BTreeMap::new();
    alloc.insert(
        Address::repeat_byte(0x55),
        GenesisAccount {
            balance: sahara_hard_cap_base_units(),
            ..GenesisAccount::default()
        },
    );
    alloc.insert(
        epoch_system_tx_sender(),
        GenesisAccount {
            balance: U256::from(EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI),
            ..GenesisAccount::default()
        },
    );

    assert_eq!(
        validate_genesis_alloc(&alloc),
        Ok(sahara_hard_cap_base_units())
    );
}
