use super::*;

#[tokio::test]
async fn boundary_unlock_credits_simplex_validator_addresses_and_conserves_balance() {
    let validators = vec![
        validators::ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: Address::repeat_byte(0x11),
        },
        validators::ValidatorEntry {
            consensus_pubkey: [0x22; 32],
            ethereum_address: Address::repeat_byte(0x22),
        },
        validators::ValidatorEntry {
            consensus_pubkey: [0x33; 32],
            ethereum_address: Address::repeat_byte(0x33),
        },
    ];
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(25_u64),
        unlock_every_epochs: 1,
        unlock_amount_per_cycle: U256::from(10_u64),
    };
    let (app, db) = setup_app_with_unlock_config(vec![], unlock_config, validators.clone());

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

    let parent = app.genesis().await;
    let (_block, _result) = app.propose(&parent, 1).await.expect("boundary propose");

    let db = db.read().unwrap();
    let community_pool_balance = account_balance(&db, COMMUNITY_POOL_ADDRESS);
    let fee_pool_balance = account_balance(&db, FEE_POOL_PRECOMPILE_ADDRESS);
    let remaining_locked = storage_value(&db, 
        COMMUNITY_POOL_ADDRESS,
        community_pool_locked_remaining_slot(),
    );
    let last_processed = storage_value(&db, 
        COMMUNITY_POOL_ADDRESS,
        community_pool_last_processed_epoch_slot(),
    );
    let current_epoch = storage_value(&db, EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot());

    assert_eq!(current_epoch, U256::from(1_u64));
    assert_eq!(community_pool_balance, U256::from(15_u64));
    assert_eq!(fee_pool_balance, U256::from(10_u64));
    assert_eq!(remaining_locked, U256::from(15_u64));
    assert_eq!(last_processed, U256::from(1_u64));

    let claim0 = storage_value(&db, 
        FEE_POOL_PRECOMPILE_ADDRESS,
        claimable_balance_slot(validators[0].ethereum_address),
    );
    let claim1 = storage_value(&db, 
        FEE_POOL_PRECOMPILE_ADDRESS,
        claimable_balance_slot(validators[1].ethereum_address),
    );
    let claim2 = storage_value(&db, 
        FEE_POOL_PRECOMPILE_ADDRESS,
        claimable_balance_slot(validators[2].ethereum_address),
    );
    assert_eq!(claim0, U256::from(4_u64));
    assert_eq!(claim1, U256::from(3_u64));
    assert_eq!(claim2, U256::from(3_u64));
    assert_eq!(claim0 + claim1 + claim2, U256::from(10_u64));
}

#[tokio::test]
async fn boundary_unlock_final_tranche_distributes_top_k_remainder() {
    let validators: Vec<_> = (1_u8..=5_u8)
        .map(|idx| validators::ValidatorEntry {
            consensus_pubkey: [idx; 32],
            ethereum_address: Address::repeat_byte(idx),
        })
        .collect();
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(4_u64),
        unlock_every_epochs: 1,
        unlock_amount_per_cycle: U256::from(10_u64),
    };
    let (app, db) = setup_app_with_unlock_config(vec![], unlock_config, validators.clone());

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

    let parent = app.genesis().await;
    let (_block, _result) = app.propose(&parent, 1).await.expect("boundary propose");

    let db = db.read().unwrap();
    assert_eq!(
        account_balance(&db, COMMUNITY_POOL_ADDRESS),
        U256::ZERO
    );
    assert_eq!(
        account_balance(&db, FEE_POOL_PRECOMPILE_ADDRESS),
        U256::from(4_u64)
    );
    assert_eq!(
        storage_value(&db, 
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot()
        ),
        U256::ZERO
    );

    for (index, validator) in validators.iter().enumerate() {
        let claim = storage_value(&db, 
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validator.ethereum_address),
        );
        let expected = if index < 4 {
            U256::from(1_u64)
        } else {
            U256::ZERO
        };
        assert_eq!(claim, expected, "validator index {index}");
    }
}

#[tokio::test]
async fn boundary_unlock_skips_non_multiple_epoch() {
    let validators = vec![
        validators::ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: Address::repeat_byte(0x11),
        },
        validators::ValidatorEntry {
            consensus_pubkey: [0x22; 32],
            ethereum_address: Address::repeat_byte(0x22),
        },
    ];
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(25_u64),
        unlock_every_epochs: 2,
        unlock_amount_per_cycle: U256::from(10_u64),
    };
    let (app, db) = setup_app_with_unlock_config(vec![], unlock_config, validators.clone());

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

    let parent = app.genesis().await;
    let (_block, _result) = app
        .propose(&parent, 1)
        .await
        .expect("propose first boundary block");

    let db = db.read().unwrap();
    let community_pool_balance = account_balance(&db, COMMUNITY_POOL_ADDRESS);
    let fee_pool_balance = account_balance(&db, FEE_POOL_PRECOMPILE_ADDRESS);
    let locked_remaining = storage_value(&db, 
        COMMUNITY_POOL_ADDRESS,
        community_pool_locked_remaining_slot(),
    );
    let last_processed = storage_value(&db, 
        COMMUNITY_POOL_ADDRESS,
        community_pool_last_processed_epoch_slot(),
    );
    let current_epoch = storage_value(&db, EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot());

    assert_eq!(current_epoch, U256::from(1_u64));
    assert_eq!(community_pool_balance, unlock_config.genesis_prefund_amount);
    assert_eq!(fee_pool_balance, U256::ZERO);
    assert_eq!(locked_remaining, unlock_config.genesis_prefund_amount);
    assert_eq!(last_processed, U256::ZERO);

    for validator in &validators {
        assert_eq!(
            storage_value(&db, 
                FEE_POOL_PRECOMPILE_ADDRESS,
                claimable_balance_slot(validator.ethereum_address),
            ),
            U256::ZERO
        );
    }
}

#[tokio::test]
async fn boundary_unlock_applies_once_on_matching_epoch() {
    let validators = vec![
        validators::ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: Address::repeat_byte(0x11),
        },
        validators::ValidatorEntry {
            consensus_pubkey: [0x22; 32],
            ethereum_address: Address::repeat_byte(0x22),
        },
    ];
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(25_u64),
        unlock_every_epochs: 2,
        unlock_amount_per_cycle: U256::from(10_u64),
    };
    let (app, db) = setup_app_with_unlock_config(vec![], unlock_config, validators.clone());

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

    let parent = app.genesis().await;
    let (first_block, _result) = app
        .propose(&parent, 1)
        .await
        .expect("propose first boundary block");
    let (_second_block, _result) = app
        .propose(&first_block, 2)
        .await
        .expect("propose second boundary block");

    {
        let db = db.read().unwrap();
        let current_epoch = storage_value(&db, EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot());
        let community_pool_balance = account_balance(&db, COMMUNITY_POOL_ADDRESS);
        let fee_pool_balance = account_balance(&db, FEE_POOL_PRECOMPILE_ADDRESS);
        let locked_remaining = storage_value(&db, 
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot(),
        );
        let last_processed = storage_value(&db, 
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot(),
        );
        let claim0 = storage_value(&db, 
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validators[0].ethereum_address),
        );
        let claim1 = storage_value(&db, 
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validators[1].ethereum_address),
        );

        assert_eq!(current_epoch, U256::from(2_u64));
        assert_eq!(community_pool_balance, U256::from(15_u64));
        assert_eq!(fee_pool_balance, U256::from(10_u64));
        assert_eq!(locked_remaining, U256::from(15_u64));
        assert_eq!(last_processed, U256::from(2_u64));
        assert_eq!(claim0, U256::from(5_u64));
        assert_eq!(claim1, U256::from(5_u64));
        assert_eq!(claim0 + claim1, U256::from(10_u64));
    }

    let (
        community_pool_before_repeat,
        fee_pool_before_repeat,
        remaining_before_repeat,
        last_processed_before_repeat,
        claim0_before_repeat,
        claim1_before_repeat,
    ) = {
        let db = db.read().unwrap();
        (
            account_balance(&db, COMMUNITY_POOL_ADDRESS),
            account_balance(&db, FEE_POOL_PRECOMPILE_ADDRESS),
            storage_value(&db, 
                COMMUNITY_POOL_ADDRESS,
                community_pool_locked_remaining_slot(),
            ),
            storage_value(&db, 
                COMMUNITY_POOL_ADDRESS,
                community_pool_last_processed_epoch_slot(),
            ),
            storage_value(&db, 
                FEE_POOL_PRECOMPILE_ADDRESS,
                claimable_balance_slot(validators[0].ethereum_address),
            ),
            storage_value(&db, 
                FEE_POOL_PRECOMPILE_ADDRESS,
                claimable_balance_slot(validators[1].ethereum_address),
            ),
        )
    };

    {
        let mut db = db.write().unwrap();
        apply_post_block_accounting(
            &mut *db,
            &PostBlockAccountingInputs {
                boundary_required: true,
                gas_used: 0,
                base_fee_per_gas: 1,
                priority_fees: U256::ZERO,
                claim_recipient: Address::ZERO,
                simplex_validators: validators.clone(),
            },
        )
        .expect("same-epoch accounting invocation must preserve unlock state");
    }

    let db = db.read().unwrap();
    assert_eq!(
        account_balance(&db, COMMUNITY_POOL_ADDRESS),
        community_pool_before_repeat
    );
    assert_eq!(
        account_balance(&db, FEE_POOL_PRECOMPILE_ADDRESS),
        fee_pool_before_repeat
    );
    assert_eq!(
        storage_value(&db, 
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot()
        ),
        remaining_before_repeat
    );
    assert_eq!(
        storage_value(&db, 
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot()
        ),
        last_processed_before_repeat
    );
    assert_eq!(
        storage_value(&db, 
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validators[0].ethereum_address),
        ),
        claim0_before_repeat
    );
    assert_eq!(
        storage_value(&db, 
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validators[1].ethereum_address),
        ),
        claim1_before_repeat
    );
}
