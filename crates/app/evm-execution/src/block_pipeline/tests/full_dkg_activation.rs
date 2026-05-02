use crate::block_pipeline::tests::*;

#[tokio::test]
async fn propose_rejects_non_boundary_full_dkg_players_mismatch_with_activation_schedule() {
    let chain_spec = Arc::new(build_test_chain_spec());
    let base_config =
        WhirlpoolEvmConfig::new(chain_spec.clone()).with_local_proposer_public_key([0x77; 32]);
    let candidate_players = default_validator_pubkeys();
    let proposer_config = base_config
        .with_current_full_dkg_output(validators_dkg::FullDkgOutputV1 {
            dealers: candidate_players.clone(),
            players: candidate_players,
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        })
        .with_activation_players_for_epoch(0, vec![[0x41; 32], [0x42; 32]]);
    let (app, _db) = setup_app_with_config(vec![], proposer_config).await;

    let parent = app.genesis().await;
    let err = app
        .propose(&parent, 1)
        .await
        .expect_err("non-boundary propose must fail-closed when activation players mismatch");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("full_dkg output.players does not match activation-resolved player set")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn propose_dkg_preflight_failure_does_not_mutate_canonical_state() {
    let (tx, recovered) = sample_evm_tx();
    let chain_spec = Arc::new(build_test_chain_spec());
    let base_config =
        WhirlpoolEvmConfig::new(chain_spec.clone()).with_local_proposer_public_key([0x77; 32]);
    let candidate_players = default_validator_pubkeys();
    let proposer_config = base_config
        .with_current_full_dkg_output(validators_dkg::FullDkgOutputV1 {
            dealers: candidate_players.clone(),
            players: candidate_players,
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        })
        .with_activation_players_for_epoch(0, vec![[0x41; 32], [0x42; 32]]);
    let (app, db) = setup_app_with_config(vec![tx], proposer_config).await;

    {
        let mut db = db.write().unwrap();
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
    }
    let pre_state = db.read().unwrap().clone();
    let pre_root = state_root_value(&pre_state);

    let parent = app.genesis().await;
    let err = app
        .propose(&parent, 1)
        .await
        .expect_err("DKG preflight must reject before canonical commit");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("full_dkg output.players does not match activation-resolved player set")),
        "unexpected error: {err:?}"
    );

    let post_state = db.read().unwrap().clone();
    assert_eq!(
        state_root_value(&post_state),
        pre_root,
        "canonical DB must remain unchanged when state-backed DKG preflight fails"
    );
}

#[tokio::test]
async fn propose_uses_runtime_registry_order_for_non_boundary_dkg_defaults() {
    let chain_spec = Arc::new(build_test_chain_spec());
    let runtime_entries = vec![
        ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: TEST_PROPOSER_FEE_RECIPIENT,
        },
        ValidatorEntry {
            consensus_pubkey: [0x77; 32],
            ethereum_address: TEST_PROPOSER_FEE_RECIPIENT,
        },
    ];
    let runtime_players = ordered_consensus_pubkeys(&runtime_entries);
    let config = WhirlpoolEvmConfig::new(chain_spec)
        .with_local_proposer_public_key([0x11; 32])
        .with_current_full_dkg_output(validators_dkg::FullDkgOutputV1 {
            dealers: runtime_players.clone(),
            players: runtime_players.clone(),
            public_polynomial: vec![],
        });
    let (app, db) = setup_app_with_config(vec![], config).await;
    {
        let mut db = db.write().unwrap();
        seed_validator_registry(&mut db, &runtime_entries);
    }

    let parent = app.genesis().await;
    let (block, _) = app
        .propose(&parent, 1)
        .await
        .expect("runtime-order DKG defaults should propose");
    let decoded =
        decode_header_extra_data(&block.extra_data).expect("canonical extra_data should decode");

    assert!(
        decoded.dkg.full_dkg.is_none(),
        "candidate matching runtime registry defaults should be omitted; a config/chainspec fallback would require FullDkg"
    );
}

#[tokio::test]
async fn verify_rejects_non_boundary_full_dkg_players_mismatch_with_activation_schedule() {
    let chain_spec = Arc::new(build_test_chain_spec());
    let base_config =
        WhirlpoolEvmConfig::new(chain_spec.clone()).with_local_proposer_public_key([0x77; 32]);
    let candidate_players = default_validator_pubkeys();
    let candidate_output = validators_dkg::FullDkgOutputV1 {
        dealers: candidate_players.clone(),
        players: candidate_players,
        public_polynomial: vec![0xaa, 0xbb, 0xcc],
    };
    let proposer_config = base_config
        .clone()
        .with_current_full_dkg_output(candidate_output.clone());
    let (app, db) = setup_app_with_config(vec![], proposer_config).await;

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.expect("non-boundary propose");

    let verifier_config = base_config
        .with_current_full_dkg_output(candidate_output)
        .with_activation_players_for_epoch(0, vec![[0x41; 32], [0x42; 32]]);
    let verifier = EvmApplication::new(
        verifier_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("non-boundary verify must fail-closed when activation players mismatch");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("full_dkg output.players does not match activation-resolved player set")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn propose_boundary_block_emits_forward_full_dkg_and_reshare_sections_when_candidate_configured(
) {
    let chain_spec = Arc::new(build_test_chain_spec());
    let base_config =
        WhirlpoolEvmConfig::new(chain_spec.clone()).with_local_proposer_public_key([0x77; 32]);
    let players = default_validator_pubkeys();
    let proposer_config =
        base_config.with_current_full_dkg_output(validators_dkg::FullDkgOutputV1 {
            dealers: players.clone(),
            players: players.clone(),
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        });
    let (app, db) = setup_app_with_config(vec![], proposer_config).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
    }

    let parent = app.genesis().await;
    let (block, _) = app
        .propose(&parent, 1)
        .await
        .expect("boundary block should propose");
    let decoded =
        decode_header_extra_data(&block.extra_data).expect("canonical extra_data should decode");

    let full_dkg = decoded
        .dkg
        .full_dkg
        .as_ref()
        .expect("boundary block should include full_dkg");
    assert_eq!(full_dkg.epoch, 2, "boundary full_dkg must target epoch E+1");
    assert_eq!(full_dkg.output.players, players);

    let reshare = decoded
        .dkg
        .reshare
        .as_ref()
        .expect("boundary block should include reshare");
    assert_eq!(
        reshare.target_epoch, 3,
        "boundary reshare must target epoch E+2"
    );
    assert_eq!(reshare.players, full_dkg.output.players);
}

#[tokio::test]
async fn verify_rejects_missing_reshare_section_on_boundary_when_candidate_configured() {
    let chain_spec = Arc::new(build_test_chain_spec());
    let base_config =
        WhirlpoolEvmConfig::new(chain_spec.clone()).with_local_proposer_public_key([0x77; 32]);
    let players = default_validator_pubkeys();
    let proposer_config =
        base_config.with_current_full_dkg_output(validators_dkg::FullDkgOutputV1 {
            dealers: players.clone(),
            players: players.clone(),
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        });
    let (app, db) = setup_app_with_config(vec![], proposer_config.clone()).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (mut block, _) = app
        .propose(&parent, 1)
        .await
        .expect("boundary block should propose");

    let mut decoded =
        decode_header_extra_data(&block.extra_data).expect("canonical extra_data must decode");
    decoded.dkg.reshare = None;
    block.extra_data =
        encode_header_extra_data(&decoded).expect("mutated canonical extra_data encodes");

    let verifier = EvmApplication::new(
        proposer_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("missing boundary reshare must be rejected");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("reshare section must be present for boundary block")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn verify_rejects_reshare_section_on_non_boundary_block() {
    let chain_spec = Arc::new(build_test_chain_spec());
    let base_config =
        WhirlpoolEvmConfig::new(chain_spec.clone()).with_local_proposer_public_key([0x77; 32]);
    let players = default_validator_pubkeys();
    let proposer_config =
        base_config.with_current_full_dkg_output(validators_dkg::FullDkgOutputV1 {
            dealers: players.clone(),
            players: players.clone(),
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        });
    let (app, db) = setup_app_with_config(vec![], proposer_config.clone()).await;

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (mut block, _) = app
        .propose(&parent, 1)
        .await
        .expect("non-boundary block should propose");

    let mut decoded =
        decode_header_extra_data(&block.extra_data).expect("canonical extra_data must decode");
    decoded.dkg.reshare = Some(validators_dkg::ReshareV1 {
        target_epoch: 1,
        players: players.clone(),
    });
    block.extra_data =
        encode_header_extra_data(&decoded).expect("mutated canonical extra_data encodes");

    let verifier = EvmApplication::new(
        proposer_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("reshare on non-boundary block must be rejected");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("reshare section is forbidden on non-boundary blocks")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn verify_rejects_full_dkg_section_when_feature_is_disabled() {
    let chain_spec = Arc::new(build_test_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_feature_enabled(false);
    let players = default_validator_pubkeys();
    let candidate_full_dkg = validators_dkg::FullDkgV1 {
        epoch: 1,
        output: validators_dkg::FullDkgOutputV1 {
            dealers: players.clone(),
            players,
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        },
    };
    let (app, db) = setup_app_with_config(vec![], base_config.clone()).await;

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (mut block, _) = app
        .propose(&parent, 1)
        .await
        .expect("non-boundary block should propose");

    block.extra_data = encode_header_extra_data(&CanonicalHeaderExtraDataV1 {
        raw_eth: Some(block.proposer_public_key.to_vec()),
        dkg: DkgHeaderSections {
            full_dkg: Some(candidate_full_dkg),
            reshare: None,
        },
    })
    .expect("canonical extra_data with forbidden full_dkg should encode");

    let verifier = EvmApplication::new(
        base_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("verify must reject full_dkg metadata when feature is disabled");

    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("full_dkg and reshare sections must be omitted")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn boundary_reshare_can_follow_epoch_pipeline_lag_from_activation_schedule() {
    let chain_spec = Arc::new(build_test_chain_spec());
    let base_config =
        WhirlpoolEvmConfig::new(chain_spec.clone()).with_local_proposer_public_key([0x77; 32]);

    let next_epoch_players = default_validator_pubkeys();
    let next_next_epoch_players = vec![[0x41; 32], [0x42; 32]];
    let proposer_config = base_config
        .with_current_full_dkg_output(validators_dkg::FullDkgOutputV1 {
            dealers: next_epoch_players.clone(),
            players: next_epoch_players.clone(),
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        })
        .with_activation_players_for_epoch(2, next_epoch_players.clone())
        .with_activation_players_for_epoch(3, next_next_epoch_players.clone());

    let (app, db) = setup_app_with_config(vec![], proposer_config.clone()).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _) = app
        .propose(&parent, 1)
        .await
        .expect("boundary block should propose");
    let decoded =
        decode_header_extra_data(&block.extra_data).expect("canonical extra_data should decode");
    let full_dkg = decoded
        .dkg
        .full_dkg
        .as_ref()
        .expect("boundary block should include full_dkg");
    let reshare = decoded
        .dkg
        .reshare
        .as_ref()
        .expect("boundary block should include reshare");

    assert_eq!(
        full_dkg.output.players, next_epoch_players,
        "full_dkg players should match next-epoch activation set"
    );
    assert_eq!(
        reshare.players, next_next_epoch_players,
        "reshare players should match next-next-epoch activation set"
    );

    let verifier = EvmApplication::new(
        proposer_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    verifier
        .verify(&parent, &block)
        .await
        .expect("boundary verify should accept activation pipeline-lag schedule");
}

#[tokio::test]
async fn propose_rejects_boundary_when_activation_schedule_missing_reshare_epoch() {
    let chain_spec = Arc::new(build_test_chain_spec());
    let base_config =
        WhirlpoolEvmConfig::new(chain_spec.clone()).with_local_proposer_public_key([0x77; 32]);
    let next_epoch_players = default_validator_pubkeys();
    let proposer_config = base_config
        .with_current_full_dkg_output(validators_dkg::FullDkgOutputV1 {
            dealers: next_epoch_players.clone(),
            players: next_epoch_players.clone(),
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        })
        .with_activation_players_for_epoch(2, next_epoch_players);

    let (app, db) = setup_app_with_config(vec![], proposer_config).await;
    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
    }

    let parent = app.genesis().await;
    let err = app
        .propose(&parent, 1)
        .await
        .expect_err("boundary propose should fail-closed when reshare epoch data is missing");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("activation resolver missing player set for epoch 3")),
        "unexpected error: {err:?}"
    );
}
