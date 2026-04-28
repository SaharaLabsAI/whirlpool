use super::*;

#[tokio::test]
async fn verify_boundary_unlock_matches_propose_state() {
    let validators = vec![
        validators_reader::ValidatorEntry {
            consensus_pubkey: [0x01; 32],
            ethereum_address: Address::repeat_byte(0x01),
        },
        validators_reader::ValidatorEntry {
            consensus_pubkey: [0x02; 32],
            ethereum_address: Address::repeat_byte(0x02),
        },
    ];
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(11_u64),
        unlock_every_epochs: 1,
        unlock_amount_per_cycle: U256::from(5_u64),
    };
    let chain_spec = Arc::new(
            build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config(
                BTreeMap::new(),
                BTreeMap::new(),
                validators.clone(),
                unlock_config,
            ),
        );
    let (app, db) =
        setup_app_with_config(vec![], WhirlpoolEvmConfig::new(chain_spec.clone())).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, 1);
        seed_community_pool_unlock_state(
            &mut db,
            unlock_config.unlock_every_epochs,
            unlock_config.unlock_amount_per_cycle,
            unlock_config.genesis_prefund_amount,
            unlock_config.genesis_prefund_amount,
        );
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _result) = app
        .propose(&parent, 1)
        .await
        .expect("propose boundary block");

    let proposer_state = db.read().unwrap().clone();
    let proposer_state_root = state_root_value(&proposer_state);
    let verifier_db = Arc::new(RwLock::new(pre_state));
    let verifier_app = EvmApplication::new(
        WhirlpoolEvmConfig::new(chain_spec),
        verifier_db.clone(),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let verify_result = verifier_app
        .verify(&parent, &block)
        .await
        .expect("verify boundary block with unlock");

    // verify() computes against an ephemeral clone and checks the computed state root
    // against the block; it does not mutate the application's backing DB.
    assert_eq!(verify_result.state_root, block.state_root);
    assert_eq!(verify_result.receipts_root, block.receipts_root);
    assert_eq!(proposer_state_root, block.state_root);

    let verifier_state = verifier_db.read().unwrap();
    assert_eq!(
        account_balance(&verifier_state, COMMUNITY_POOL_ADDRESS),
        unlock_config.genesis_prefund_amount
    );
    assert_eq!(
        storage_value(
            &verifier_state,
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot()
        ),
        unlock_config.genesis_prefund_amount
    );
    assert_eq!(
        storage_value(
            &verifier_state,
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot()
        ),
        U256::ZERO
    );
}

#[tokio::test]
async fn boundary_block_receipts_and_gas_are_user_only() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
    }

    let parent = app.genesis().await;
    let (block, result) = app
        .propose(&parent, 1)
        .await
        .expect("propose boundary block");
    let receipts = app.pending_receipts();

    assert_eq!(block.transactions.len(), 1);
    assert_eq!(receipts.len(), 1);
    assert_eq!(result.receipt_count, 1);
    assert_eq!(
        block.gas_used,
        receipts
            .last()
            .expect("must have receipt")
            .cumulative_gas_used
    );
}

#[tokio::test]
async fn verify_accepts_boundary_block_with_user_only_transactions() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
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
    let (block, _) = app
        .propose(&parent, 1)
        .await
        .expect("propose boundary block");

    let verifier = EvmApplication::new(
        WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec())),
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );

    assert!(verifier.verify(&parent, &block).await.is_ok());
}
