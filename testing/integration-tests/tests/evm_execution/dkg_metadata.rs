use std::sync::{Arc, RwLock};

use app_evm_execution::{EvmAppError, EvmApplication, WhirlpoolEvmConfig};
use app_evm_state::InMemoryStateDb;
use app_traits::{traits::Application, NoopTxSource};
use chainspec::build_sahara_chain_spec;
use evm_precompiles::{
    current_epoch_slot, epoch_blocks_slot, epoch_system_tx_sender, next_epoch_block_slot,
    EPOCH_BLOCKS_DEFAULT, EPOCH_PRECOMPILE_ADDRESS, EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI,
};
use revm::primitives::U256;
use validators_dkg::{
    decode_extra_data, encode_canonical_extra_data, CanonicalExtraDataV1, FullDkgOutputV1,
};

fn dkg_config_with_output(output: FullDkgOutputV1) -> WhirlpoolEvmConfig {
    WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()))
        .with_current_full_dkg_output(output)
}

fn dkg_app_with_config(
    config: WhirlpoolEvmConfig,
) -> (
    EvmApplication<InMemoryStateDb>,
    Arc<RwLock<InMemoryStateDb>>,
) {
    let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
    let app = EvmApplication::new(config, db.clone(), Arc::new(NoopTxSource));
    (app, db)
}

fn seed_epoch_boundary_state(db: &mut InMemoryStateDb, next_epoch_block: u64) {
    db.insert_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        current_epoch_slot(),
        U256::from(0_u64),
    );
    db.insert_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        epoch_blocks_slot(),
        U256::from(EPOCH_BLOCKS_DEFAULT),
    );
    db.insert_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        next_epoch_block_slot(),
        U256::from(next_epoch_block),
    );
    db.insert_account(
        epoch_system_tx_sender(),
        revm::state::AccountInfo {
            balance: U256::from(EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI),
            nonce: 0,
            ..Default::default()
        },
    );
}

#[tokio::test]
async fn non_boundary_dkg_candidate_is_included_and_verifies() {
    let base_config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let players = base_config.simplex_consensus_public_keys();
    let output = FullDkgOutputV1 {
        dealers: vec![[0x44; 32]],
        players,
        public_polynomial: vec![0x99, 0xaa],
    };
    let (app, _) = dkg_app_with_config(dkg_config_with_output(output.clone()));

    let genesis = app.genesis().await;
    let (block, _) = app.propose(&genesis, 1).await.expect("propose");
    let decoded = decode_extra_data(&block.extra_data).expect("canonical extra_data decodes");

    assert_eq!(decoded.full_dkg.expect("full_dkg included").output, output);
    assert!(decoded.reshare.is_none());
    app.verify(&genesis, &block)
        .await
        .expect("matching non-boundary dkg block verifies");
}

#[tokio::test]
async fn non_boundary_unchanged_baseline_dkg_candidate_is_omitted_and_verifies() {
    let base_config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let players = base_config.simplex_consensus_public_keys();
    let output = FullDkgOutputV1 {
        dealers: players.clone(),
        players,
        public_polynomial: vec![],
    };
    let (app, _) = dkg_app_with_config(dkg_config_with_output(output));

    let genesis = app.genesis().await;
    let (block, _) = app.propose(&genesis, 1).await.expect("propose");
    let decoded = decode_extra_data(&block.extra_data).expect("canonical extra_data decodes");

    assert!(decoded.full_dkg.is_none());
    assert!(decoded.reshare.is_none());
    app.verify(&genesis, &block)
        .await
        .expect("omitted unchanged non-boundary dkg block verifies");
}

#[tokio::test]
async fn non_boundary_omit_uses_latest_committed_history_across_raw_only_intermediate_block() {
    let base_config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let players = base_config.simplex_consensus_public_keys();
    let output = FullDkgOutputV1 {
        dealers: vec![[0x44; 32]],
        players,
        public_polynomial: vec![0x99, 0xaa],
    };
    let (app, db) = dkg_app_with_config(dkg_config_with_output(output));

    let genesis = app.genesis().await;
    let (block1, _) = app.propose(&genesis, 1).await.expect("block1 propose");
    let decoded1 = decode_extra_data(&block1.extra_data).expect("block1 extra_data decodes");
    assert!(
        decoded1.full_dkg.is_some(),
        "block1 establishes DKG history"
    );
    {
        let guard = db.read().unwrap();
        app.store_finalized_block(&block1, &*guard)
            .expect("store block1 history");
    }

    let (block2, _) = app.propose(&block1, 2).await.expect("block2 propose");
    let decoded2 = decode_extra_data(&block2.extra_data).expect("block2 extra_data decodes");
    assert!(
        decoded2.full_dkg.is_none(),
        "block2 is raw-only intermediate"
    );
    {
        let guard = db.read().unwrap();
        app.store_finalized_block(&block2, &*guard)
            .expect("store raw-only block2 history");
    }

    let (block3, _) = app.propose(&block2, 3).await.expect("block3 propose");
    let decoded3 = decode_extra_data(&block3.extra_data).expect("block3 extra_data decodes");
    assert!(
        decoded3.full_dkg.is_none(),
        "block3 must scan past raw-only block2 and omit unchanged FullDKG"
    );
    app.verify(&block2, &block3)
        .await
        .expect("raw-only history omission verifies");
}

#[tokio::test]
async fn boundary_dkg_candidate_includes_full_dkg_and_reshare_and_verifies() {
    let base_config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let players = base_config.simplex_consensus_public_keys();
    let output = FullDkgOutputV1 {
        dealers: players.clone(),
        players: players.clone(),
        public_polynomial: vec![0xaa, 0xbb, 0xcc],
    };
    let config = base_config.with_current_full_dkg_output(output);
    let (app, db) = dkg_app_with_config(config.clone());
    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1);
    }
    let pre_state = db.read().unwrap().clone();

    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.expect("boundary propose");
    let decoded = decode_extra_data(&block.extra_data).expect("canonical extra_data decodes");
    let full_dkg = decoded.full_dkg.expect("boundary full_dkg included");
    let reshare = decoded.reshare.expect("boundary reshare included");

    assert_eq!(full_dkg.epoch, 2);
    assert_eq!(full_dkg.output.players, players);
    assert_eq!(reshare.target_epoch, 3);
    assert_eq!(reshare.players, full_dkg.output.players);

    let verifier = EvmApplication::new(
        config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(NoopTxSource),
    );
    verifier
        .verify(&parent, &block)
        .await
        .expect("boundary dkg block verifies");
}

#[tokio::test]
async fn disabled_feature_rejects_dkg_metadata() {
    let base_config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let players = base_config.simplex_consensus_public_keys();
    let output = FullDkgOutputV1 {
        dealers: vec![[0x44; 32]],
        players,
        public_polynomial: vec![0x99],
    };
    let proposer_config = base_config
        .clone()
        .with_current_full_dkg_output(output.clone());
    let (app, db) = dkg_app_with_config(proposer_config);
    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.expect("propose dkg metadata");

    let verifier_config = base_config
        .with_current_full_dkg_output(output)
        .with_full_dkg_feature_enabled(false);
    let verifier = EvmApplication::new(
        verifier_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(NoopTxSource),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("disabled feature must reject dkg metadata");

    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("must be omitted when full_dkg feature is disabled"))
    );
}

#[tokio::test]
async fn verify_rejects_mismatched_dkg_candidate_payload() {
    let base_config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let players = base_config.simplex_consensus_public_keys();
    let output = FullDkgOutputV1 {
        dealers: vec![[0x44; 32]],
        players,
        public_polynomial: vec![0x99],
    };
    let config = base_config.with_current_full_dkg_output(output);
    let (app, db) = dkg_app_with_config(config.clone());
    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (mut block, _) = app.propose(&parent, 1).await.expect("propose");

    let mut decoded = decode_extra_data(&block.extra_data).expect("canonical extra_data decodes");
    decoded
        .full_dkg
        .as_mut()
        .expect("full_dkg included")
        .output
        .public_polynomial
        .push(0xee);
    block.extra_data = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: decoded.raw_eth,
        full_dkg: decoded.full_dkg,
        reshare: decoded.reshare,
    })
    .expect("mutated extra_data encodes");

    let verifier = EvmApplication::new(
        config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(NoopTxSource),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("mismatched dkg payload must be rejected");

    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("full_dkg payload mismatch"))
    );
}
