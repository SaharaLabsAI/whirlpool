use super::*;

#[tokio::test]
async fn boundary_block_keeps_user_transactions_only() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx.clone()]).await;

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

    assert_eq!(block.transactions, vec![tx]);
    assert_eq!(result.receipt_count, 1);
}

#[tokio::test]
async fn propose_excludes_reserved_epoch_namespace_transaction() {
    let reserved_tx = sample_reserved_epoch_namespace_tx(0, 2_000_000_000);
    let (user_tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![reserved_tx, user_tx.clone()]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 10, EPOCH_BLOCKS_DEFAULT);
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
        .expect("propose should skip reserved namespace transaction");

    assert_eq!(block.transactions, vec![user_tx]);
    assert_eq!(result.receipt_count, 1);
    assert_eq!(app.pending_receipts().len(), 1);
}

#[tokio::test]
async fn propose_keeps_non_zero_value_system_advance_epoch_tx_outside_reserved_namespace() {
    let near_miss_tx = sample_epoch_system_call_tx(0, 2_000_000_000, U256::from(1_u64));
    let (app, db) = setup_app(vec![near_miss_tx.clone()]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 10, EPOCH_BLOCKS_DEFAULT);
    }

    let parent = app.genesis().await;
    let (block, result) = app
        .propose(&parent, 1)
        .await
        .expect("non-zero near miss should not be filtered as reserved namespace");

    assert_eq!(block.transactions, vec![near_miss_tx]);
    assert_eq!(result.receipt_count, 1);
    assert_eq!(app.pending_receipts().len(), 1);
}

#[tokio::test]
async fn boundary_block_system_call_advances_epoch_state_once() {
    let (app, db) = setup_app(vec![]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
    }
    {
        let db = db.read().unwrap();
        let boundary_state =
            evm_precompiles::load_epoch_boundary_state(&*db).expect("load boundary state");
        assert_eq!(boundary_state.next_epoch_block, 1);
    }

    let parent = app.genesis().await;
    let (boundary_block, _) = app
        .propose(&parent, 1)
        .await
        .expect("propose boundary block");
    {
        let db = db.read().unwrap();
        assert_eq!(
            db.get_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot()),
            U256::from(1_u64)
        );
    }
    let (_next_block, _) = app
        .propose(&boundary_block, 2)
        .await
        .expect("propose non-boundary block");

    let db = db.read().unwrap();
    assert_eq!(
        db.get_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot()),
        U256::from(1_u64)
    );
    assert_eq!(
        db.get_storage(EPOCH_PRECOMPILE_ADDRESS, next_epoch_block_slot()),
        U256::from(1_u64 + EPOCH_BLOCKS_DEFAULT)
    );
}
