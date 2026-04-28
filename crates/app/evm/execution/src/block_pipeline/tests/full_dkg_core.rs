use super::*;

#[test]
fn latest_committed_full_dkg_scans_backwards_past_raw_eth_only_blocks() {
    #[derive(Default)]
    struct MockHistory {
        blocks: BTreeMap<u64, Vec<u8>>,
    }

    impl validators_dkg::DkgHistory for MockHistory {
        type Error = String;

        fn full_dkg_at_height(&self, height: u64) -> Result<Option<Vec<u8>>, Self::Error> {
            Ok(self.blocks.get(&height).cloned())
        }
    }

    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec);
    let players = config.simplex_consensus_public_keys();
    let full_dkg = FullDkgV1 {
        epoch: 1,
        output: validators_dkg::FullDkgOutputV1 {
            dealers: players.clone(),
            players: players.clone(),
            public_polynomial: vec![1, 2, 3, 4],
        },
    };

    let extra_with_full_dkg = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(vec![0x11; 32]),
        full_dkg: Some(full_dkg.clone()),
        reshare: None,
    })
    .expect("encode full_dkg");
    let extra_raw_only = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(vec![0x11; 32]),
        full_dkg: None,
        reshare: None,
    })
    .expect("encode raw only");

    let mut history = MockHistory::default();
    history.blocks.insert(0, extra_with_full_dkg);
    history.blocks.insert(1, extra_raw_only.clone());
    history.blocks.insert(2, extra_raw_only);

    let resolved = validators_dkg::latest_committed_full_dkg(&history, 2)
        .expect("scan should succeed")
        .expect("full_dkg should resolve from earlier block");
    assert_eq!(resolved, full_dkg);

    assert!(
        !full_dkg_should_be_included(
            &config.simplex_consensus_public_keys(),
            Some(&resolved),
            &full_dkg
        ),
        "unchanged baseline must not force redundant FullDkg inclusion"
    );
}

#[test]
fn full_dkg_trigger_includes_when_only_dealers_change() {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec);
    let players = config.simplex_consensus_public_keys();

    let previous = FullDkgV1 {
        epoch: 3,
        output: validators_dkg::FullDkgOutputV1 {
            dealers: vec![[0x11; 32]],
            players: players.clone(),
            public_polynomial: vec![0xaa, 0xbb],
        },
    };
    let candidate = FullDkgV1 {
        epoch: 3,
        output: validators_dkg::FullDkgOutputV1 {
            dealers: vec![[0x22; 32]],
            players,
            public_polynomial: vec![0xaa, 0xbb],
        },
    };

    assert!(
        full_dkg_should_be_included(
            &config.simplex_consensus_public_keys(),
            Some(&previous),
            &candidate
        ),
        "dealer-only changes must trigger FullDkg inclusion"
    );
}

#[tokio::test]
async fn verify_rejects_full_dkg_payload_mismatch_against_candidate() {
    let (tx, recovered) = sample_evm_tx();
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone());
    let players = base_config.simplex_consensus_public_keys();
    let candidate_output = validators_dkg::FullDkgOutputV1 {
        dealers: players.clone(),
        players: players.clone(),
        public_polynomial: vec![0xaa, 0xbb, 0xcc],
    };
    let proposer_config = base_config
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0)
        .with_current_full_dkg_output(candidate_output.clone());
    let (app, db) = setup_app_with_config(vec![tx], proposer_config.clone()).await;

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
    let parent = app.genesis().await;
    let (mut block, _) = app.propose(&parent, 1).await.unwrap();

    let mut decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Strict)
        .expect("canonical extra_data must decode");
    decoded
        .full_dkg
        .as_mut()
        .expect("proposed block should include full_dkg")
        .output
        .public_polynomial
        .push(0xff);
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
        .expect_err("mismatched full_dkg payload must be rejected");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("full_dkg payload mismatch")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn verify_rejects_full_dkg_when_candidate_is_not_configured() {
    let (tx, recovered) = sample_evm_tx();
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let (app, db) = setup_app_with_config(vec![tx], config.clone()).await;

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
    let parent = app.genesis().await;
    let (mut block, _) = app.propose(&parent, 1).await.unwrap();

    let players = config.simplex_consensus_public_keys();
    block.extra_data = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(block.proposer_public_key.to_vec()),
        full_dkg: Some(FullDkgV1 {
            epoch: 0,
            output: validators_dkg::FullDkgOutputV1 {
                dealers: players.clone(),
                players,
                public_polynomial: vec![0x01],
            },
        }),
        reshare: None,
    })
    .expect("canonical extra_data with full_dkg");

    let verifier = EvmApplication::new(
        config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("unexpected full_dkg without configured candidate must be rejected");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("must be omitted when no full_dkg candidate is configured")),
        "unexpected error: {err:?}"
    );
}
