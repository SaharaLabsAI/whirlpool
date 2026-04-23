use super::*;

#[tokio::test]
async fn verify_accepts_valid_block() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.unwrap();

    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec).with_local_proposer_public_key([0x77; 32]);
    let pre_db = Arc::new(RwLock::new(pre_state));
    let source = Arc::new(MockTxSource { txs: vec![] });
    let verifier_app = EvmApplication::new(config, pre_db, source);

    assert!(verifier_app.verify(&parent, &block).await.is_ok());
}

#[tokio::test]
async fn verify_rejects_block_with_mismatched_base_fee_per_gas() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (mut block, _) = app.propose(&parent, 1).await.unwrap();
    block.base_fee_per_gas += 1;

    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec).with_local_proposer_public_key([0x77; 32]);
    let verifier = EvmApplication::new(
        config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("verify must reject mismatched base fee");

    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("base fee mismatch")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn verify_accepts_legacy_extra_data_before_strict_height() {
    let strict_height = 2;
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let proposer_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(strict_height);
    let (app, db) = setup_app_with_config(vec![], proposer_config.clone()).await;
    let pre_state = db.read().unwrap().clone();

    let parent = app.genesis().await;
    let (mut block, _) = app.propose(&parent, 1).await.unwrap();
    block.extra_data = legacy_proposer_extra_data_bytes(block.proposer_public_key);

    let verifier = EvmApplication::new(
        proposer_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );

    assert!(
        verifier.verify(&parent, &block).await.is_ok(),
        "legacy extra_data must remain accepted before strict-height boundary"
    );
}

#[tokio::test]
async fn verify_rejects_legacy_extra_data_at_or_after_strict_height() {
    let strict_height = 2;
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let proposer_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(strict_height);
    let (app, db) = setup_app_with_config(vec![], proposer_config.clone()).await;

    let genesis = app.genesis().await;
    let (parent, _) = app.propose(&genesis, 1).await.unwrap();
    let pre_state = db.read().unwrap().clone();
    let (mut block, _) = app.propose(&parent, strict_height).await.unwrap();
    block.extra_data = legacy_proposer_extra_data_bytes(block.proposer_public_key);

    let verifier = EvmApplication::new(
        proposer_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("legacy extra_data must be rejected at/after strict height");

    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("failed to decode block extra_data")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn verify_accepts_block_with_precompile_proxy_transaction() {
    let proxy_address = Address::with_last_byte(0xaa);
    let (tx, recovered) = sample_proxy_precompile_withdraw_tx(proxy_address);
    let (app, db) = setup_app(vec![tx]).await;
    let claimable = U256::from(5_u64);

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
        let mut proxy_info = revm::state::AccountInfo::default();
        proxy_info.set_code(Bytecode::new_raw(precompile_proxy_runtime_bytecode()));
        db.insert_account(proxy_address, proxy_info);
        let fee_pool_info = revm::state::AccountInfo {
            balance: claimable,
            ..Default::default()
        };
        db.insert_account(FEE_POOL_PRECOMPILE_ADDRESS, fee_pool_info);
        db.insert_storage(
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(proxy_address),
            claimable,
        );
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.unwrap();
    let current_balance = account_balance(&db.read().unwrap(), proxy_address);
    assert_eq!(current_balance, claimable);

    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec).with_local_proposer_public_key([0x77; 32]);
    let pre_db = Arc::new(RwLock::new(pre_state));
    let source = Arc::new(MockTxSource { txs: vec![] });
    let verifier_app = EvmApplication::new(config, pre_db, source);

    assert!(verifier_app.verify(&parent, &block).await.is_ok());
}

#[tokio::test]
async fn verify_rejects_fee_recipient_that_conflicts_with_genesis_mapping() {
    let proposer_public_key = [0x11; 32];
    let expected_fee_recipient = Address::repeat_byte(0x22);
    let mut validator_fee_recipients = BTreeMap::new();
    validator_fee_recipients.insert(proposer_public_key, expected_fee_recipient);

    let chain_spec = Arc::new(build_sahara_chain_spec_with_alloc_and_fee_recipients(
        BTreeMap::new(),
        validator_fee_recipients,
    ));
    let proposer_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key(proposer_public_key);

    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app_with_config(vec![tx], proposer_config).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (mut block, _) = app.propose(&parent, 1).await.unwrap();
    block.proposer_fee_recipient = Address::repeat_byte(0x77).into_array();

    let verifier_config =
        WhirlpoolEvmConfig::new(chain_spec).with_local_proposer_public_key([0x55; 32]);
    let verifier_app = EvmApplication::new(
        verifier_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );

    let err = verifier_app
        .verify(&parent, &block)
        .await
        .expect_err("genesis mapping should reject mismatched fee recipient");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(_)),
        "expected invalid block error, got {err:?}"
    );
}
