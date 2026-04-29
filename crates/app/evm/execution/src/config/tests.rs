use crate::config::WhirlpoolEvmConfig;
use alloy_primitives::Address;
use chainspec::{build_sahara_chain_spec, SAHARA_CHAIN_ID};
use evm_precompiles::{
    COMMUNITY_POOL_ADDRESS, EPOCH_PRECOMPILE_ADDRESS, FEE_POOL_PRECOMPILE_ADDRESS,
    VALIDATORS_PRECOMPILE_ADDRESS,
};
use reth_chainspec::EthereumHardforks;
use reth_evm::{ConfigureEvm, Evm, EvmFactory, NextBlockEnvAttributes};
use reth_primitives_traits::Header;
use revm::{database::EmptyDB, primitives::B256};
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
fn activation_schedule_uses_supplied_state_backed_default_players() {
    let default_players = vec![[0x11; 32], [0x22; 32]];
    let schedule = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()))
        .validator_activation_schedule_for_default_players(default_players.clone());
    let resolved = schedule
        .resolve_players_for_epoch(42)
        .expect("default activation players should resolve");
    assert_eq!(resolved, default_players);
}

#[test]
fn activation_players_can_be_epoch_overridden() {
    let default_players = vec![[0x11; 32], [0x22; 32]];
    let players_epoch_7 = vec![[0x77; 32], [0x78; 32]];
    let schedule = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()))
        .with_activation_players_for_epoch(7, players_epoch_7.clone())
        .validator_activation_schedule_for_default_players(default_players);

    assert_eq!(
        schedule.resolve_players_for_epoch(7).ok(),
        Some(players_epoch_7)
    );
    assert!(schedule.resolve_players_for_epoch(8).is_err());
}
