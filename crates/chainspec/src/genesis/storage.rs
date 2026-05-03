use alloy_genesis::GenesisAccount;
use alloy_primitives::Address;
use alloy_primitives::U256;
use evm_precompiles::community_pool_last_processed_epoch_storage_slot;
use evm_precompiles::community_pool_locked_remaining_storage_slot;
use evm_precompiles::community_pool_unlock_amount_per_cycle_storage_slot;
use evm_precompiles::community_pool_unlock_every_epochs_storage_slot;
use evm_precompiles::current_epoch_storage_slot;
use evm_precompiles::encode_epoch_start_block_storage_value;
use evm_precompiles::encode_u256_storage_value;
use evm_precompiles::encode_u64_storage_value;
use evm_precompiles::epoch_blocks_storage_slot;
use evm_precompiles::epoch_system_tx_sender;
use evm_precompiles::next_epoch_block_storage_slot;
use evm_precompiles::COMMUNITY_POOL_ADDRESS;
use evm_precompiles::EPOCH_BLOCKS_DEFAULT;
use evm_precompiles::EPOCH_PRECOMPILE_ADDRESS;
use evm_precompiles::EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI;
use std::collections::BTreeMap;
use validators_reader::encode_validator_registry_storage;
use validators_reader::ValidatorEntry;
use validators_reader::SIMPLEX_VALIDATORS_REGISTRY;

use crate::community_pool::CommunityPoolUnlockConfig;
use crate::native_token::NativeTokenError;

pub fn seed_validator_registry(
    alloc: &mut BTreeMap<Address, GenesisAccount>,
    simplex_validators: &[ValidatorEntry],
) {
    if simplex_validators.is_empty() {
        return;
    }

    let account = alloc
        .entry(SIMPLEX_VALIDATORS_REGISTRY)
        .or_insert_with(|| GenesisAccount {
            balance: U256::ZERO,
            ..GenesisAccount::default()
        });
    account.storage = Some(encode_validator_registry_storage(simplex_validators));
}

pub fn seed_epoch_precompile_genesis_state(alloc: &mut BTreeMap<Address, GenesisAccount>) {
    let account = alloc
        .entry(EPOCH_PRECOMPILE_ADDRESS)
        .or_insert_with(|| GenesisAccount {
            balance: U256::ZERO,
            ..GenesisAccount::default()
        });
    let storage = account.storage.get_or_insert_with(BTreeMap::new);
    storage.insert(current_epoch_storage_slot(), encode_u64_storage_value(0));
    storage.insert(
        epoch_blocks_storage_slot(),
        encode_u64_storage_value(EPOCH_BLOCKS_DEFAULT),
    );
    storage.insert(
        next_epoch_block_storage_slot(),
        encode_u64_storage_value(EPOCH_BLOCKS_DEFAULT),
    );
    storage.insert(
        evm_precompiles::epoch_start_block_storage_slot(0),
        encode_epoch_start_block_storage_value(0),
    );

    let sender = epoch_system_tx_sender();
    let sender_account = alloc.entry(sender).or_insert_with(|| GenesisAccount {
        balance: U256::ZERO,
        ..GenesisAccount::default()
    });
    sender_account.balance = sender_account
        .balance
        .checked_add(U256::from(EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI))
        .expect("epoch system sender balance seed should not overflow");
    sender_account.nonce = Some(0);
}

pub fn seed_community_pool_genesis_state(
    alloc: &mut BTreeMap<Address, GenesisAccount>,
    config: CommunityPoolUnlockConfig,
) -> Result<(), NativeTokenError> {
    let has_config = !config.genesis_prefund_amount.is_zero()
        || config.unlock_every_epochs > 0
        || !config.unlock_amount_per_cycle.is_zero();
    if !has_config {
        return Ok(());
    }

    let account = alloc
        .entry(COMMUNITY_POOL_ADDRESS)
        .or_insert_with(|| GenesisAccount {
            balance: U256::ZERO,
            ..GenesisAccount::default()
        });

    account.balance = account
        .balance
        .checked_add(config.genesis_prefund_amount)
        .ok_or(NativeTokenError::SupplyOverflow)?;

    let storage = account.storage.get_or_insert_with(BTreeMap::new);
    storage.insert(
        community_pool_unlock_every_epochs_storage_slot(),
        encode_u64_storage_value(config.unlock_every_epochs),
    );
    storage.insert(
        community_pool_unlock_amount_per_cycle_storage_slot(),
        encode_u256_storage_value(config.unlock_amount_per_cycle),
    );
    storage.insert(
        community_pool_locked_remaining_storage_slot(),
        encode_u256_storage_value(config.genesis_prefund_amount),
    );
    storage.insert(
        community_pool_last_processed_epoch_storage_slot(),
        encode_u64_storage_value(0),
    );

    Ok(())
}
