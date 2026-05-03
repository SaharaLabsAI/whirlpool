use crate::config::WhirlpoolEvmConfig;
use alloy_primitives::Address;
use chainspec::genesis::build_sahara_chain_spec;
use chainspec::SAHARA_CHAIN_ID;
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

#[test]
fn proposer_context_delegate_preserves_local_key() {
    let local_key = [0x42; 32];
    let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()))
        .with_local_proposer_public_key(local_key);

    assert_eq!(config.local_proposer_public_key(), local_key);
    assert_eq!(config.proposer_context().local_public_key(), local_key);
}

#[test]
fn full_dkg_feature_gate_delegate_preserves_default_and_override() {
    let default_config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    assert!(default_config.full_dkg_feature_enabled());
    assert!(default_config.dkg_transition().feature_gate().enabled());

    let disabled = default_config.with_full_dkg_feature_enabled(false);
    assert!(!disabled.full_dkg_feature_enabled());
    assert!(!disabled.dkg_transition().feature_gate().enabled());
}

#[test]
fn current_full_dkg_candidate_delegate_preserves_candidate_input() {
    let candidate = validators_dkg::FullDkgOutputV1 {
        dealers: vec![[0x11; 32]],
        players: vec![[0x22; 32], [0x23; 32]],
        public_polynomial: vec![0xaa, 0xbb],
    };
    let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()))
        .with_current_full_dkg_output(candidate.clone());

    assert_eq!(config.current_full_dkg_output(), Some(&candidate));
    assert_eq!(
        config.dkg_transition().current_candidate().output(),
        Some(&candidate)
    );
}

#[test]
fn dkg_activation_overrides_delegate_keeps_state_backed_defaults_as_input() {
    let default_players = vec![[0x11; 32], [0x22; 32]];
    let override_players = vec![[0x33; 32], [0x44; 32]];

    let default_schedule = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()))
        .dkg_transition()
        .activation_schedule_for_default_players(default_players.clone());
    assert_eq!(
        default_schedule.resolve_players_for_epoch(1).ok(),
        Some(default_players.clone())
    );

    let config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()))
        .with_activation_players_for_epoch(9, override_players.clone());
    let schedule_from_config =
        config.validator_activation_schedule_for_default_players(default_players.clone());
    let schedule_from_owner = config
        .dkg_transition()
        .activation_schedule_for_default_players(default_players);

    assert_eq!(
        schedule_from_config.resolve_players_for_epoch(9).ok(),
        Some(override_players.clone())
    );
    assert_eq!(
        schedule_from_owner.resolve_players_for_epoch(9).ok(),
        Some(override_players)
    );
    assert!(schedule_from_config.resolve_players_for_epoch(10).is_err());
    assert!(schedule_from_owner.resolve_players_for_epoch(10).is_err());
}
