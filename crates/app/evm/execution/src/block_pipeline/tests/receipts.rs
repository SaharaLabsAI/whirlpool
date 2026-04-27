use super::*;

#[tokio::test]
async fn propose_cache_isolated_for_same_height_different_parent() {
    let (app, _db) = setup_app(vec![]).await;
    let parent = app.genesis().await;
    let (first_block, _) = app.propose(&parent, 1).await.expect("first propose");

    let mut alternate_parent = parent.clone();
    alternate_parent.state_root[0] ^= 0x01;
    let (second_block, _) = app
        .propose(&alternate_parent, 1)
        .await
        .expect("second propose with alternate parent");

    assert_eq!(first_block.parent_id, parent.compute_id());
    assert_eq!(second_block.parent_id, alternate_parent.compute_id());
    assert_ne!(first_block.parent_id, second_block.parent_id);
}

#[tokio::test]
async fn verify_rejects_parent_id_mismatch() {
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
    let (block, _) = app.propose(&parent, 1).await.expect("propose block");
    let mut wrong_parent = parent.clone();
    wrong_parent.state_root[0] ^= 0x01;

    let err = app
        .verify(&wrong_parent, &block)
        .await
        .expect_err("verify must reject mismatched parent id");
    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
}

#[tokio::test]
async fn store_finalized_block_retains_receipts_when_store_fails() {
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
    let (block, _) = app.propose(&parent, 1).await.expect("propose block");

    struct FailingBlockStorage;
    impl BlockStorage for FailingBlockStorage {
        fn store_block(
            &self,
            _block: &EvmBlock,
            _receipts: &[Receipt],
        ) -> Result<(), state::BlockStorageError> {
            Err(state::BlockStorageError::Database(
                "injected persistence failure".into(),
            ))
        }

        fn get_block_by_number(
            &self,
            _number: u64,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_block_by_hash(
            &self,
            _hash: B256,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_receipts_by_block(
            &self,
            _number: u64,
        ) -> Result<Option<Vec<Receipt>>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_latest_block_number(&self) -> Result<Option<u64>, state::BlockStorageError> {
            Ok(None)
        }
    }

    let err = app
        .store_finalized_block(&block, &FailingBlockStorage)
        .expect_err("finalize persistence failure should return error");
    assert!(matches!(err, EvmAppError::State(_)));
    assert_eq!(app.pending_receipts().len(), 1);
    assert!(app.has_staged_receipts_for(block.compute_id()));
}

#[tokio::test]
async fn store_finalized_block_rejects_receipts_for_mismatched_cached_block() {
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
    let (block, _) = app.propose(&parent, 1).await.expect("propose block");
    let staged_block_id = block.compute_id();
    let mut mismatched_block = block.clone();
    mismatched_block.parent_id[0] ^= 0x01;

    #[derive(Default)]
    struct CountingStorage {
        calls: Mutex<usize>,
    }

    impl BlockStorage for CountingStorage {
        fn store_block(
            &self,
            _block: &EvmBlock,
            _receipts: &[Receipt],
        ) -> Result<(), state::BlockStorageError> {
            let mut calls = self.calls.lock().unwrap();
            *calls += 1;
            Ok(())
        }

        fn get_block_by_number(
            &self,
            _number: u64,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_block_by_hash(
            &self,
            _hash: B256,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_receipts_by_block(
            &self,
            _number: u64,
        ) -> Result<Option<Vec<Receipt>>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_latest_block_number(&self) -> Result<Option<u64>, state::BlockStorageError> {
            Ok(None)
        }
    }

    let storage = CountingStorage::default();
    let err = app
        .store_finalized_block(&mismatched_block, &storage)
        .expect_err("mismatched staged receipts must be rejected");
    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
    assert_eq!(*storage.calls.lock().unwrap(), 0);
    assert!(app.has_staged_receipts_for(staged_block_id));
}

#[tokio::test]
async fn store_finalized_block_stores_and_clears_receipts() {
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
    let (block, _) = app.propose(&parent, 1).await.unwrap();

    #[derive(Default)]
    struct MockBlockStorage {
        stored: Mutex<Vec<(EvmBlock, Vec<Receipt>)>>,
    }

    impl BlockStorage for MockBlockStorage {
        fn store_block(
            &self,
            block: &EvmBlock,
            receipts: &[Receipt],
        ) -> Result<(), state::BlockStorageError> {
            self.stored
                .lock()
                .unwrap()
                .push((block.clone(), receipts.to_vec()));
            Ok(())
        }

        fn get_block_by_number(
            &self,
            _number: u64,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_block_by_hash(
            &self,
            _hash: B256,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_receipts_by_block(
            &self,
            _number: u64,
        ) -> Result<Option<Vec<Receipt>>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_latest_block_number(&self) -> Result<Option<u64>, state::BlockStorageError> {
            Ok(None)
        }
    }

    let storage = MockBlockStorage::default();
    app.store_finalized_block(&block, &storage).unwrap();

    let stored = storage.stored.lock().unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].0.height, 1);
    assert_eq!(stored[0].1.len(), 1);
    assert!(app.pending_receipts().is_empty());
    assert!(app.staged_receipts_is_empty());
}
