use super::{
    WhirlpoolEvmConfig, DEFAULT_PROPOSER_FEE_RECIPIENT, VALIDATOR_FEE_RECIPIENTS_REGISTRY,
};
use crate::EpochBoundaryHook;
use alloy_primitives::Address;
use chainspec::{
    build_sahara_chain_spec, build_sahara_chain_spec_with_alloc_and_fee_recipients, SAHARA_CHAIN_ID,
};
use evm_precompiles::{
    COMMUNITY_POOL_ADDRESS, EPOCH_PRECOMPILE_ADDRESS, FEE_POOL_PRECOMPILE_ADDRESS,
    VALIDATORS_PRECOMPILE_ADDRESS,
};
use reth_chainspec::EthereumHardforks;
use reth_evm::{ConfigureEvm, Evm, EvmFactory, NextBlockEnvAttributes};
use reth_primitives_traits::Header;
use revm::{database::EmptyDB, primitives::B256};
use std::collections::BTreeMap;
use std::sync::Arc;

#[test]
fn test_evm_config_chain_spec() {
    let spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(spec.clone());

    assert!(Arc::ptr_eq(config.chain_spec(), &spec));
    assert_eq!(config.chain_spec().chain.id(), SAHARA_CHAIN_ID);
    assert_eq!(config.chain_spec().genesis.gas_limit, 30_000_000);
    assert!(config.chain_spec().is_cancun_active_at_timestamp(0));
}

#[test]
fn test_evm_config_exposes_factory_and_assembler() {
    let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));

    let _factory: &<WhirlpoolEvmConfig as ConfigureEvm>::BlockExecutorFactory =
        config.block_executor_factory();
    let _assembler: &<WhirlpoolEvmConfig as ConfigureEvm>::BlockAssembler =
        config.block_assembler();
}

#[test]
fn test_evm_config_installs_whirlpool_precompiles() {
    let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let env = config
        .next_evm_env(
            &Header::default(),
            &NextBlockEnvAttributes {
                timestamp: 1,
                suggested_fee_recipient: Address::ZERO,
                prev_randao: B256::ZERO,
                gas_limit: 30_000_000,
                parent_beacon_block_root: Some(B256::ZERO),
                withdrawals: None,
                extra_data: Default::default(),
            },
        )
        .expect("next EVM env");
    let evm = config.evm_factory().create_evm(EmptyDB::default(), env);

    assert!(evm.precompiles().get(&COMMUNITY_POOL_ADDRESS).is_some());
    assert!(evm
        .precompiles()
        .get(&FEE_POOL_PRECOMPILE_ADDRESS)
        .is_some());
    assert!(evm
        .precompiles()
        .get(&VALIDATORS_PRECOMPILE_ADDRESS)
        .is_some());
    assert!(evm.precompiles().get(&EPOCH_PRECOMPILE_ADDRESS).is_some());
}

#[test]
fn test_default_fee_recipient_is_non_zero() {
    let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));

    assert_eq!(config.fee_recipient(), DEFAULT_PROPOSER_FEE_RECIPIENT);
    assert_ne!(config.fee_recipient(), Address::ZERO);
}

#[test]
fn test_fee_recipient_mapping_roundtrip_in_genesis_registry() {
    let local_proposer_public_key = [0x11; 32];
    let custom = Address::repeat_byte(0x44);
    let mut validator_fee_recipients = BTreeMap::new();
    validator_fee_recipients.insert(local_proposer_public_key, custom);

    let spec = Arc::new(build_sahara_chain_spec_with_alloc_and_fee_recipients(
        BTreeMap::new(),
        validator_fee_recipients,
    ));
    let config = WhirlpoolEvmConfig::new(spec.clone())
        .with_local_proposer_public_key(local_proposer_public_key);

    assert_eq!(config.fee_recipient(), custom);
    assert!(spec
        .genesis
        .alloc
        .contains_key(&VALIDATOR_FEE_RECIPIENTS_REGISTRY));
}

#[test]
fn test_full_dkg_strict_height_defaults_to_zero() {
    let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    assert_eq!(config.full_dkg_strict_height(), 0);
}

#[test]
fn activation_players_default_to_simplex_registry() {
    let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let expected = config.simplex_consensus_public_keys();
    let resolved = config
        .activation_players_for_epoch(42)
        .expect("default activation players should resolve");
    assert_eq!(resolved, expected);
}

#[test]
fn activation_players_can_be_epoch_overridden() {
    let players_epoch_7 = vec![[0x77; 32], [0x78; 32]];
    let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()))
        .with_activation_players_for_epoch(7, players_epoch_7.clone());

    assert_eq!(
        config.activation_players_for_epoch(7),
        Some(players_epoch_7)
    );
    assert_eq!(config.activation_players_for_epoch(8), None);
}

#[test]
fn epoch_boundary_hook_defaults_to_precompile_semantics() {
    let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    assert_eq!(
        config.epoch_boundary_hook(),
        EpochBoundaryHook::PrecompileSemanticsV1
    );
}

#[test]
fn epoch_boundary_hook_can_be_overridden() {
    let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()))
        .with_epoch_boundary_hook(EpochBoundaryHook::PrecompileSemanticsV1);
    assert_eq!(
        config.epoch_boundary_hook(),
        EpochBoundaryHook::PrecompileSemanticsV1
    );
}
