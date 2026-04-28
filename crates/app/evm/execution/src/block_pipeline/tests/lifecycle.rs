use std::sync::{Arc, RwLock};

use super::*;
use crate::error::EvmAppError;
use app_evm_state::InMemoryStateDb;
use app_traits::traits::Application;

fn build_noop_app() -> EvmApplication<InMemoryStateDb> {
    let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));
    let evm_config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let tx_source = Arc::new(app_traits::NoopTxSource);
    EvmApplication::new(evm_config, state_db, tx_source)
}

#[tokio::test]
async fn test_execute_empty_block() {
    let app = build_noop_app();
    let genesis = app.genesis().await;

    let (_block, result) = app.propose(&genesis, 1).await.unwrap();

    assert_eq!(result.gas_used, 0);
    assert_eq!(result.receipt_count, 0);
}

#[tokio::test]
async fn test_state_root_computation() {
    let app = build_noop_app();
    let genesis = app.genesis().await;

    let (block, result) = app.propose(&genesis, 1).await.unwrap();

    assert_eq!(block.state_root, result.state_root);
}

#[tokio::test]
async fn test_reconstruct_header_for_verify() {
    let app = build_noop_app();
    let genesis = app.genesis().await;

    let (block1, _result) = app.propose(&genesis, 1).await.unwrap();

    let verify_result = app.verify(&genesis, &block1).await;
    assert!(verify_result.is_ok());
}

#[tokio::test]
async fn test_full_propose_verify_cycle() {
    let (encoded_tx, alice_addr) = sample_evm_tx();
    let (proposer_app, state_db) = setup_app(vec![encoded_tx]).await;

    {
        let mut db = state_db.write().unwrap();
        let account_info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(alice_addr, account_info);
    }

    let pre_state_snapshot = state_db.read().unwrap().clone();

    let genesis = proposer_app.genesis().await;
    let (block, execution_result) = proposer_app
        .propose(&genesis, 1)
        .await
        .expect("Propose should succeed");

    assert_eq!(block.height, 1);
    assert_eq!(block.transactions.len(), 1);
    assert!(block.gas_used > 0);
    assert_eq!(block.gas_used, execution_result.gas_used);

    let validator_db = Arc::new(RwLock::new(pre_state_snapshot));
    let empty_source = Arc::new(MockTxSource { txs: vec![] });
    let validator_app = EvmApplication::new(
        WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec())),
        validator_db,
        empty_source,
    );

    let verify_result = validator_app.verify(&genesis, &block).await;
    assert!(
        verify_result.is_ok(),
        "Verification failed: {:?}",
        verify_result.err()
    );
}

#[tokio::test]
async fn test_propose_verify_success() {
    let app = build_noop_app();

    let genesis = app.genesis().await;
    let (block1, _result) = app.propose(&genesis, 1).await.unwrap();

    let verify_result = app.verify(&genesis, &block1).await;
    assert!(verify_result.is_ok());
}

#[tokio::test]
async fn test_state_root_mismatch_returns_evm_app_error() {
    let app = build_noop_app();

    let genesis = app.genesis().await;
    let (block1, _result) = app.propose(&genesis, 1).await.unwrap();

    let mut tampered_block = block1.clone();
    tampered_block.state_root[0] ^= 0x01;

    let app_verify = app.verify(&genesis, &tampered_block).await;
    assert!(matches!(
        app_verify,
        Err(EvmAppError::StateRootMismatch { .. })
    ));
}

#[tokio::test]
async fn test_genesis_to_verify() {
    let app = build_noop_app();

    let genesis = app.genesis().await;
    let (block1, _result) = app.propose(&genesis, 1).await.unwrap();

    let verify_result = app.verify(&genesis, &block1).await;
    assert!(verify_result.is_ok());
}

#[tokio::test]
async fn test_propose_verify_state_root_consistency() {
    let app = build_noop_app();

    let genesis = app.genesis().await;
    let (block1, _result1) = app.propose(&genesis, 1).await.unwrap();
    let (block2, _result2) = app.propose(&block1, 2).await.unwrap();

    let verify1 = app.verify(&genesis, &block1).await.unwrap();
    let verify2 = app.verify(&block1, &block2).await.unwrap();

    assert_eq!(verify1.state_root, block1.state_root);
    assert_eq!(verify2.state_root, block2.state_root);
    assert_eq!(block1.state_root, block2.state_root);
}

#[tokio::test]
async fn test_multi_block_state_accumulation() {
    let app = build_noop_app();

    let genesis = app.genesis().await;
    let (block1, _result1) = app.propose(&genesis, 1).await.unwrap();
    let (block2, _result2) = app.propose(&block1, 2).await.unwrap();

    assert_eq!(genesis.height, 0);
    assert_eq!(block1.height, 1);
    assert_eq!(block2.height, 2);

    assert_eq!(block1.parent_id, genesis.compute_id());
    assert_eq!(block2.parent_id, block1.compute_id());
    assert_eq!(genesis.state_root, block1.state_root);
    assert_eq!(block1.state_root, block2.state_root);
}

#[tokio::test]
async fn test_failed_verify_does_not_corrupt_state() {
    let app = build_noop_app();

    let genesis = app.genesis().await;
    let (block1, _result1) = app.propose(&genesis, 1).await.unwrap();

    let mut tampered_block1 = block1.clone();
    tampered_block1.state_root[0] ^= 0x01;

    let failed_verify = app.verify(&genesis, &tampered_block1).await;
    assert!(failed_verify.is_err());

    let block2_proposal = app.propose(&genesis, 2).await;
    assert!(block2_proposal.is_ok());

    let (block2, _result2) = block2_proposal.unwrap();
    assert_eq!(block2.parent_id, genesis.compute_id());

    let verify_block2 = app.verify(&genesis, &block2).await;
    assert!(verify_block2.is_ok());
}
