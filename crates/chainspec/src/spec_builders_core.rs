use alloy_genesis::{Genesis, GenesisAccount};
use alloy_primitives::{Address, U256};
use evm_precompiles::{
    community_pool_last_processed_epoch_storage_slot, community_pool_locked_remaining_storage_slot,
    community_pool_unlock_amount_per_cycle_storage_slot,
    community_pool_unlock_every_epochs_storage_slot, current_epoch_storage_slot,
    encode_epoch_start_block_storage_value, encode_u256_storage_value, encode_u64_storage_value,
    epoch_blocks_storage_slot, epoch_system_tx_sender, next_epoch_block_storage_slot,
    COMMUNITY_POOL_ADDRESS, EPOCH_BLOCKS_DEFAULT, EPOCH_PRECOMPILE_ADDRESS,
    EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI,
};
use reth_chainspec::{Chain, ChainSpec, ChainSpecBuilder};
use std::collections::BTreeMap;
use validators_reader::{
    encode_validator_registry_storage, ValidatorEntry, SIMPLEX_VALIDATORS_REGISTRY,
};

use crate::{validate_genesis_alloc, CommunityPoolUnlockConfig, NativeTokenError, SAHARA_CHAIN_ID};

pub fn try_build_sahara_chain_spec_with_alloc_and_validators_and_community_pool_unlock_config(
    mut alloc: BTreeMap<Address, GenesisAccount>,
    simplex_validators: Vec<ValidatorEntry>,
    community_pool_unlock_config: CommunityPoolUnlockConfig,
) -> Result<ChainSpec, NativeTokenError> {
    if !simplex_validators.is_empty() {
        let account = alloc
            .entry(SIMPLEX_VALIDATORS_REGISTRY)
            .or_insert_with(|| GenesisAccount {
                balance: U256::ZERO,
                ..GenesisAccount::default()
            });
        account.storage = Some(encode_validator_registry_storage(&simplex_validators));
    }

    if community_pool_unlock_config.is_unlock_enabled() && simplex_validators.is_empty() {
        return Err(NativeTokenError::CommunityPoolUnlockRequiresValidators);
    }

    seed_epoch_precompile_genesis_state(&mut alloc);
    seed_community_pool_genesis_state(&mut alloc, community_pool_unlock_config)?;

    validate_genesis_alloc(&alloc)?;

    Ok(ChainSpecBuilder::default()
        .chain(Chain::from_id(SAHARA_CHAIN_ID))
        .genesis(Genesis {
            gas_limit: 30_000_000,
            difficulty: U256::ZERO,
            alloc,
            ..Default::default()
        })
        .cancun_activated()
        .build())
}

fn seed_epoch_precompile_genesis_state(alloc: &mut BTreeMap<Address, GenesisAccount>) {
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

fn seed_community_pool_genesis_state(
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
