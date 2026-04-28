use super::*;

#[tokio::test]
async fn propose_rejects_non_boundary_full_dkg_players_mismatch_with_activation_schedule() {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let candidate_players = base_config.simplex_consensus_public_keys();
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
async fn verify_rejects_non_boundary_full_dkg_players_mismatch_with_activation_schedule() {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let candidate_players = base_config.simplex_consensus_public_keys();
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
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let players = base_config.simplex_consensus_public_keys();
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
    let decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Strict)
        .expect("canonical extra_data should decode");

    let full_dkg = decoded
        .full_dkg
        .as_ref()
        .expect("boundary block should include full_dkg");
    assert_eq!(full_dkg.epoch, 2, "boundary full_dkg must target epoch E+1");
    assert_eq!(full_dkg.output.players, players);

    let reshare = decoded
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
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let players = base_config.simplex_consensus_public_keys();
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

    let mut decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Strict)
        .expect("canonical extra_data must decode");
    decoded.reshare = None;
    block.extra_data =
        encode_canonical_extra_data(&decoded).expect("mutated canonical extra_data encodes");

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
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let players = base_config.simplex_consensus_public_keys();
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

    let mut decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Strict)
        .expect("canonical extra_data must decode");
    decoded.reshare = Some(validators_dkg::ReshareV1 {
        target_epoch: 1,
        players: players.clone(),
    });
    block.extra_data =
        encode_canonical_extra_data(&decoded).expect("mutated canonical extra_data encodes");

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
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_feature_enabled(false)
        .with_full_dkg_strict_height(0);
    let players = base_config.simplex_consensus_public_keys();
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

    block.extra_data = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(block.proposer_public_key.to_vec()),
        full_dkg: Some(candidate_full_dkg),
        reshare: None,
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
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);

    let next_epoch_players = base_config.simplex_consensus_public_keys();
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
    let decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Strict)
        .expect("canonical extra_data should decode");
    let full_dkg = decoded
        .full_dkg
        .as_ref()
        .expect("boundary block should include full_dkg");
    let reshare = decoded
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
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let next_epoch_players = base_config.simplex_consensus_public_keys();
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
