use alloy_genesis::GenesisAccount;
use alloy_primitives::{Address, U256};
use evm_precompiles::epoch_system_tx_sender;
use std::collections::BTreeMap;

pub const SAHARA_DECIMALS: u8 = 18;
pub const SAHARA_HARD_CAP_TOKENS: u64 = 10_000_000_000;
pub const SAHARA_HARD_CAP_BASE_UNITS_U128: u128 = 10_000_000_000_000_000_000_000_000_000;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NativeTokenError {
    #[error("native token supply overflow while summing genesis allocations")]
    SupplyOverflow,
    #[error("native token hard cap exceeded: total {total} > cap {hard_cap}")]
    HardCapExceeded { total: U256, hard_cap: U256 },
}

pub fn sahara_hard_cap_base_units() -> U256 {
    U256::from(SAHARA_HARD_CAP_BASE_UNITS_U128)
}

pub fn total_allocated_supply(
    alloc: &BTreeMap<Address, GenesisAccount>,
) -> Result<U256, NativeTokenError> {
    alloc.values().try_fold(U256::ZERO, |total, account| {
        total
            .checked_add(account.balance)
            .ok_or(NativeTokenError::SupplyOverflow)
    })
}

pub fn validate_genesis_alloc(
    alloc: &BTreeMap<Address, GenesisAccount>,
) -> Result<U256, NativeTokenError> {
    let total = alloc.iter().try_fold(U256::ZERO, |total, (address, account)| {
        if *address == epoch_system_tx_sender() {
            return Ok(total);
        }
        total
            .checked_add(account.balance)
            .ok_or(NativeTokenError::SupplyOverflow)
    })?;
    let hard_cap = sahara_hard_cap_base_units();
    if total > hard_cap {
        Err(NativeTokenError::HardCapExceeded { total, hard_cap })
    } else {
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        epoch_system_tx_sender, sahara_hard_cap_base_units, total_allocated_supply,
        validate_genesis_alloc,
        NativeTokenError, SAHARA_DECIMALS, SAHARA_HARD_CAP_TOKENS,
    };
    use alloy_genesis::GenesisAccount;
    use alloy_primitives::{Address, U256};
    use evm_precompiles::EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI;
    use std::collections::BTreeMap;

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
}
