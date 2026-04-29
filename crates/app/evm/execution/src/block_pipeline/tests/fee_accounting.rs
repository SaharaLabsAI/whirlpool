use super::*;

#[tokio::test]
async fn propose_routes_priority_fees_to_fee_pool_not_proposer() {
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

    let parent = app.genesis().await;
    let (block, _result) = app.propose(&parent, 1).await.unwrap();
    let burned_amount = U256::from(block.gas_used) * U256::from(block.base_fee_per_gas);
    let expected_priority_fees =
        U256::from(block.gas_used) * U256::from(2_000_000_000u64 - block.base_fee_per_gas);
    let claim_slot = claimable_balance_slot(TEST_PROPOSER_FEE_RECIPIENT);

    let db = db.read().unwrap();
    let community_pool_balance = account_balance(&db, COMMUNITY_POOL_ADDRESS);
    let fee_pool_balance = account_balance(&db, FEE_POOL_PRECOMPILE_ADDRESS);
    let fee_recipient_balance = account_balance(&db, TEST_PROPOSER_FEE_RECIPIENT);
    let claimable = storage_value(&db, FEE_POOL_PRECOMPILE_ADDRESS, claim_slot);

    assert_eq!(community_pool_balance, burned_amount);
    assert_eq!(fee_pool_balance, expected_priority_fees);
    assert_eq!(claimable, expected_priority_fees);
    assert_eq!(fee_recipient_balance, U256::ZERO);
    assert_eq!(
        block.proposer_fee_recipient,
        TEST_PROPOSER_FEE_RECIPIENT.into_array()
    );
}

#[tokio::test]
async fn propose_uses_final_cumulative_gas_used_for_block_gas_and_burned_fee_credit() {
    let (tx0, recovered0) = sample_evm_tx_with_nonce(0, Address::with_last_byte(2));
    let (tx1, recovered1) = (3u8..=u8::MAX)
        .map(|byte| sample_evm_tx_with_nonce(0, Address::with_last_byte(byte)))
        .find(|(_, recovered)| *recovered != recovered0)
        .expect("must find a second sender");
    let (app, db) = setup_app(vec![tx0, tx1]).await;

    {
        let mut db = db.write().unwrap();
        for recovered in [recovered0, recovered1] {
            let info = revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            };
            db.insert_account(recovered, info);
        }
    }

    let parent = app.genesis().await;
    let (block, _result) = app.propose(&parent, 1).await.unwrap();
    let receipts = app.pending_receipts();
    assert_eq!(receipts.len(), 2, "expected two successful tx receipts");

    let expected_gas_used = receipts.last().expect("has receipts").cumulative_gas_used;
    assert_eq!(
        block.gas_used, expected_gas_used,
        "block gas used should equal final cumulative gas"
    );

    let burned_amount = U256::from(block.gas_used) * U256::from(block.base_fee_per_gas);
    let expected_priority_fees =
        U256::from(block.gas_used) * U256::from(2_000_000_000u64 - block.base_fee_per_gas);
    let claim_slot = claimable_balance_slot(TEST_PROPOSER_FEE_RECIPIENT);

    let db = db.read().unwrap();
    let community_pool_balance = account_balance(&db, COMMUNITY_POOL_ADDRESS);
    let fee_pool_balance = account_balance(&db, FEE_POOL_PRECOMPILE_ADDRESS);
    let claimable = storage_value(&db, FEE_POOL_PRECOMPILE_ADDRESS, claim_slot);

    assert_eq!(
        community_pool_balance, burned_amount,
        "community pool burn credit should use corrected block gas used"
    );
    assert_eq!(
        fee_pool_balance, expected_priority_fees,
        "fee-pool sink should be credited exactly once by execution beneficiary"
    );
    assert_eq!(claimable, expected_priority_fees);
}

#[tokio::test]
async fn burned_fee_credit_preserves_community_pool_unlock_storage() {
    let validators = vec![validators_reader::ValidatorEntry {
        consensus_pubkey: [0x11; 32],
        ethereum_address: Address::repeat_byte(0x11),
    }];
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(50_u64),
        unlock_every_epochs: 2,
        unlock_amount_per_cycle: U256::from(7_u64),
    };
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app_with_unlock_config(vec![tx], unlock_config, validators);
    let initial_community_pool_balance = U256::from(50_u64);
    let expected_locked_remaining = U256::from(42_u64);
    let expected_last_processed_epoch = U256::from(7_u64);

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 10, EPOCH_BLOCKS_DEFAULT);
        seed_community_pool_unlock_state(
            &mut db,
            unlock_config.unlock_every_epochs,
            unlock_config.unlock_amount_per_cycle,
            expected_locked_remaining,
            initial_community_pool_balance,
        );
        db.insert_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot(),
            expected_last_processed_epoch,
        );
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
    let (block, _result) = app
        .propose(&parent, 1)
        .await
        .expect("propose non-boundary block with burned fee credit");
    let burned_amount = U256::from(block.gas_used) * U256::from(block.base_fee_per_gas);

    let db = db.read().unwrap();
    let community_pool_balance = account_balance(&db, COMMUNITY_POOL_ADDRESS);
    assert_eq!(
        storage_value(&db, EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot()),
        U256::ZERO,
        "non-boundary block should not advance epoch state"
    );
    assert_eq!(
        community_pool_balance,
        initial_community_pool_balance + burned_amount,
        "burned fee credit should only increase community pool balance"
    );
    assert_eq!(
        storage_value(
            &db,
            COMMUNITY_POOL_ADDRESS,
            community_pool_unlock_every_epochs_slot()
        ),
        U256::from(unlock_config.unlock_every_epochs)
    );
    assert_eq!(
        storage_value(
            &db,
            COMMUNITY_POOL_ADDRESS,
            community_pool_unlock_amount_per_cycle_slot()
        ),
        unlock_config.unlock_amount_per_cycle
    );
    assert_eq!(
        storage_value(
            &db,
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot()
        ),
        expected_locked_remaining
    );
    assert_eq!(
        storage_value(
            &db,
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot()
        ),
        expected_last_processed_epoch
    );
}
