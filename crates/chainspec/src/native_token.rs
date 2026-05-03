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
    #[error("community-pool unlock schedule requires at least one simplex validator")]
    CommunityPoolUnlockRequiresValidators,
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
    let total = alloc
        .iter()
        .try_fold(U256::ZERO, |total, (address, account)| {
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
