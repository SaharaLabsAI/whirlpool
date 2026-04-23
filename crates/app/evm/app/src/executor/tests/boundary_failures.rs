use super::*;

#[tokio::test]
async fn verify_rejects_reserved_epoch_namespace_transaction() {
    let reserved_tx = sample_reserved_epoch_namespace_tx(0, 2_000_000_000);
    let (app, db) = setup_app(vec![]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 10, EPOCH_BLOCKS_DEFAULT);
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let expected_base_fee_per_gas = calc_next_block_base_fee(
        parent.gas_used,
        BLOCK_GAS_LIMIT,
        parent.base_fee_per_gas,
        BaseFeeParams::ethereum(),
    );
    let block = EvmBlock {
        height: 1,
        parent_id: parent.compute_id(),
        state_root: parent.state_root,
        transactions_root: ordered_trie_root_with_encoder(
            std::slice::from_ref(&reserved_tx),
            |tx, out| out.put_slice(tx),
        )
        .0,
        receipts_root: EMPTY_ROOT_HASH.0,
        proposer_public_key: parent.proposer_public_key,
        proposer_fee_recipient: parent.proposer_fee_recipient,
        extra_data: parent.extra_data.clone(),
        gas_used: 0,
        base_fee_per_gas: expected_base_fee_per_gas,
        timestamp: parent.timestamp + 12,
        transactions: vec![reserved_tx],
    };

    let verifier = EvmApplication::new(
        WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec())),
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );

    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("reserved epoch namespace tx must be invalid");
    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
    assert!(
        err.to_string()
            .contains("reserved epoch boundary namespace transaction"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn propose_rejects_when_required_boundary_system_call_fails() {
    let (app, db) = setup_app(vec![]).await;

    {
        let mut db = db.write().unwrap();
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            current_epoch_slot(),
            U256::from(u64::MAX),
        );
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            epoch_blocks_slot(),
            U256::from(EPOCH_BLOCKS_DEFAULT),
        );
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            next_epoch_block_slot(),
            U256::from(1_u64),
        );
    }

    let parent = app.genesis().await;
    let err = app
        .propose(&parent, 1)
        .await
        .expect_err("boundary system call failure must fail proposal");
    assert!(matches!(err, EvmAppError::Execution(_)));
}

#[tokio::test]
async fn verify_rejects_when_required_boundary_system_call_fails() {
    let (app, db) = setup_app(vec![]).await;

    {
        let mut db = db.write().unwrap();
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            current_epoch_slot(),
            U256::from(u64::MAX),
        );
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            epoch_blocks_slot(),
            U256::from(EPOCH_BLOCKS_DEFAULT),
        );
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            next_epoch_block_slot(),
            U256::from(1_u64),
        );
    }

    let parent = app.genesis().await;
    let expected_base_fee_per_gas = calc_next_block_base_fee(
        parent.gas_used,
        BLOCK_GAS_LIMIT,
        parent.base_fee_per_gas,
        BaseFeeParams::ethereum(),
    );
    let boundary_block = EvmBlock {
        height: 1,
        parent_id: parent.compute_id(),
        state_root: parent.state_root,
        transactions_root: EMPTY_ROOT_HASH.0,
        receipts_root: EMPTY_ROOT_HASH.0,
        proposer_public_key: parent.proposer_public_key,
        proposer_fee_recipient: parent.proposer_fee_recipient,
        extra_data: parent.extra_data.clone(),
        gas_used: 0,
        base_fee_per_gas: expected_base_fee_per_gas,
        timestamp: parent.timestamp + 12,
        transactions: vec![],
    };

    let err = app
        .verify(&parent, &boundary_block)
        .await
        .expect_err("boundary system call must fail verification");
    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
}
