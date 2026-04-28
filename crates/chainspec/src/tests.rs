use crate::{
    build_sahara_chain_spec, build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators,
    build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config,
    sahara_hard_cap_base_units, try_build_sahara_chain_spec_with_alloc,
    try_build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config,
    try_simplex_validators_from_chain_spec, CommunityPoolUnlockConfig, NativeTokenError,
    SAHARA_CHAIN_ID,
};
use alloy_genesis::GenesisAccount;
use alloy_primitives::{address, Address, U256};
use evm_precompiles::{
    community_pool_last_processed_epoch_storage_slot, community_pool_locked_remaining_storage_slot,
    community_pool_unlock_amount_per_cycle_storage_slot,
    community_pool_unlock_every_epochs_storage_slot, current_epoch_storage_slot,
    encode_epoch_start_block_storage_value, encode_u256_storage_value, encode_u64_storage_value,
    epoch_blocks_storage_slot, epoch_system_tx_sender, next_epoch_block_storage_slot,
    COMMUNITY_POOL_ADDRESS, EPOCH_BLOCKS_DEFAULT, EPOCH_PRECOMPILE_ADDRESS,
    EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI,
};
use reth_chainspec::EthereumHardforks;
use std::collections::BTreeMap;
use validators_reader::{ValidatorEntry, SIMPLEX_VALIDATORS_REGISTRY};

#[test]
fn test_build_sahara_chain_spec_values() {
    let spec = build_sahara_chain_spec();

    assert_eq!(spec.chain.id(), SAHARA_CHAIN_ID);
    assert_eq!(spec.genesis.gas_limit, 30_000_000);
    assert!(spec.is_cancun_active_at_timestamp(0));
}

#[test]
fn chain_spec_builder_writes_epoch_precompile_genesis_state() {
    let spec = build_sahara_chain_spec();
    let account = spec
        .genesis
        .alloc
        .get(&EPOCH_PRECOMPILE_ADDRESS)
        .expect("epoch precompile account");
    let storage = account.storage.as_ref().expect("epoch storage");

    assert_eq!(
        storage.get(&current_epoch_storage_slot()),
        Some(&encode_u64_storage_value(0))
    );
    assert_eq!(
        storage.get(&epoch_blocks_storage_slot()),
        Some(&encode_u64_storage_value(EPOCH_BLOCKS_DEFAULT))
    );
    assert_eq!(
        storage.get(&next_epoch_block_storage_slot()),
        Some(&encode_u64_storage_value(EPOCH_BLOCKS_DEFAULT))
    );
    assert_eq!(
        storage.get(&evm_precompiles::epoch_start_block_storage_slot(0)),
        Some(&encode_epoch_start_block_storage_value(0))
    );
}

#[test]
fn chain_spec_builder_seeds_epoch_system_sender_balance() {
    let spec = build_sahara_chain_spec();
    let sender = epoch_system_tx_sender();
    let account = spec
        .genesis
        .alloc
        .get(&sender)
        .expect("epoch system sender account");
    assert_eq!(
        account.balance,
        U256::from(EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI)
    );
    assert_eq!(account.nonce, Some(0));
}

#[test]
fn chain_spec_builder_writes_validator_registry() {
    let validators = vec![
        ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: address!("0x0000000000000000000000000000000000000011"),
        },
        ValidatorEntry {
            consensus_pubkey: [0x22; 32],
            ethereum_address: address!("0x0000000000000000000000000000000000000022"),
        },
    ];
    let spec = build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
        BTreeMap::new(),
        BTreeMap::new(),
        validators,
    );

    assert!(spec
        .genesis
        .alloc
        .contains_key(&SIMPLEX_VALIDATORS_REGISTRY));
}

#[test]
fn chain_spec_reader_matches_written_validator_registry() {
    let validators = vec![
        ValidatorEntry {
            consensus_pubkey: [0x33; 32],
            ethereum_address: address!("0x0000000000000000000000000000000000000033"),
        },
        ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: address!("0x0000000000000000000000000000000000000011"),
        },
    ];
    let spec = build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
        BTreeMap::new(),
        BTreeMap::new(),
        validators.clone(),
    );

    let decoded = try_simplex_validators_from_chain_spec(&spec).expect("decode validators");
    assert_eq!(decoded, validators);
}

#[test]
fn validator_registry_encoding_is_independent_of_fee_recipient_registry() {
    let validator_key = [0xaa; 32];
    let fee_recipient = address!("0x00000000000000000000000000000000000000aa");
    let simplex_validators = vec![ValidatorEntry {
        consensus_pubkey: validator_key,
        ethereum_address: address!("0x00000000000000000000000000000000000000bb"),
    }];
    let mut fee_recipients = BTreeMap::new();
    fee_recipients.insert(validator_key, fee_recipient);

    let spec = build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
        BTreeMap::new(),
        fee_recipients,
        simplex_validators,
    );

    assert!(spec
        .genesis
        .alloc
        .contains_key(&app_evm_execution::VALIDATOR_FEE_RECIPIENTS_REGISTRY));
    assert!(spec
        .genesis
        .alloc
        .contains_key(&SIMPLEX_VALIDATORS_REGISTRY));
    assert_ne!(
        app_evm_execution::VALIDATOR_FEE_RECIPIENTS_REGISTRY,
        SIMPLEX_VALIDATORS_REGISTRY
    );
}

#[test]
fn chain_spec_builder_prefunds_community_pool_and_seeds_unlock_state() {
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(123_u64),
        unlock_every_epochs: 4,
        unlock_amount_per_cycle: U256::from(25_u64),
    };
    let validators = vec![ValidatorEntry {
        consensus_pubkey: [0x11; 32],
        ethereum_address: address!("0x0000000000000000000000000000000000000011"),
    }];

    let spec = build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config(
        BTreeMap::new(),
        BTreeMap::new(),
        validators,
        unlock_config,
    );

    let account = spec
        .genesis
        .alloc
        .get(&COMMUNITY_POOL_ADDRESS)
        .expect("community pool account");
    assert_eq!(account.balance, unlock_config.genesis_prefund_amount);

    let storage = account.storage.as_ref().expect("community pool storage");
    assert_eq!(
        storage.get(&community_pool_unlock_every_epochs_storage_slot()),
        Some(&encode_u64_storage_value(unlock_config.unlock_every_epochs))
    );
    assert_eq!(
        storage.get(&community_pool_unlock_amount_per_cycle_storage_slot()),
        Some(&encode_u256_storage_value(
            unlock_config.unlock_amount_per_cycle
        ))
    );
    assert_eq!(
        storage.get(&community_pool_locked_remaining_storage_slot()),
        Some(&encode_u256_storage_value(
            unlock_config.genesis_prefund_amount
        ))
    );
    assert_eq!(
        storage.get(&community_pool_last_processed_epoch_storage_slot()),
        Some(&encode_u64_storage_value(0))
    );
}

#[test]
fn unlock_enabled_without_simplex_validators_is_rejected() {
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(123_u64),
        unlock_every_epochs: 4,
        unlock_amount_per_cycle: U256::from(25_u64),
    };

    assert_eq!(
        try_build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config(
            BTreeMap::new(),
            BTreeMap::new(),
            Vec::new(),
            unlock_config,
        ),
        Err(NativeTokenError::CommunityPoolUnlockRequiresValidators)
    );
}

#[test]
fn test_try_build_sahara_chain_spec_with_alloc_rejects_over_cap() {
    let mut alloc = BTreeMap::new();
    let total = sahara_hard_cap_base_units() + U256::from(1u64);
    alloc.insert(
        Address::repeat_byte(0x55),
        GenesisAccount {
            balance: total,
            ..GenesisAccount::default()
        },
    );

    assert_eq!(
        try_build_sahara_chain_spec_with_alloc(alloc),
        Err(NativeTokenError::HardCapExceeded {
            total,
            hard_cap: sahara_hard_cap_base_units(),
        })
    );
}
