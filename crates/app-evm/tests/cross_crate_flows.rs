use std::sync::{Arc, RwLock};

use app::{ApplicationAdapter, NoopTxSource, traits::Application};
use app_evm::executor::EvmApplication;
use app_evm::{EvmAppError, WhirlpoolEvmConfig, build_sahara_chain_spec};
use consensus::{ConsensusError, traits::ConsensusApp};
use state_memory::InMemoryStateDb;

fn build_app() -> EvmApplication<InMemoryStateDb> {
    let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));
    let evm_config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let tx_source = Arc::new(NoopTxSource);
    EvmApplication::new(evm_config, state_db, tx_source)
}

fn build_adapter() -> ApplicationAdapter<EvmApplication<InMemoryStateDb>> {
    ApplicationAdapter::new(build_app())
}

#[tokio::test]
async fn test_propose_verify_success() {
    let app = build_app();

    let genesis = app.genesis().await;
    let (block1, _result) = app.propose(&genesis, 1).await.unwrap();

    let verify_result = app.verify(&genesis, &block1).await;
    assert!(verify_result.is_ok());
}

#[tokio::test]
async fn test_state_root_mismatch() {
    let app = build_app();
    let adapter = ApplicationAdapter::new(app.clone());

    let genesis = app.genesis().await;
    let (block1, _result) = app.propose(&genesis, 1).await.unwrap();

    let mut tampered_block = block1.clone();
    tampered_block.state_root[0] ^= 0x01;

    let app_verify = app.verify(&genesis, &tampered_block).await;
    assert!(matches!(
        app_verify,
        Err(EvmAppError::StateRootMismatch { .. })
    ));

    let adapter_verify = adapter.verify(&genesis, &tampered_block).await;
    assert!(matches!(
        adapter_verify,
        Err(ConsensusError::InvalidBlock(_))
    ));
}

#[tokio::test]
async fn test_genesis_to_verify() {
    let app = build_app();

    let genesis = app.genesis().await;
    let (block1, _result) = app.propose(&genesis, 1).await.unwrap();

    let verify_result = app.verify(&genesis, &block1).await;
    assert!(verify_result.is_ok());
}

#[tokio::test]
async fn test_error_propagation_through_adapter() {
    let adapter = build_adapter();

    let genesis = ConsensusApp::genesis(&adapter).await;
    let block1 = ConsensusApp::propose(&adapter, &genesis, 1)
        .await
        .expect("adapter propose should return Some");

    let mut tampered_block = block1.clone();
    tampered_block.state_root[0] ^= 0x01;

    let verify_result = ConsensusApp::verify(&adapter, &genesis, &tampered_block).await;
    assert!(matches!(
        verify_result,
        Err(ConsensusError::InvalidBlock(_))
    ));
}

#[tokio::test]
async fn test_propose_verify_state_root_consistency() {
    let app = build_app();

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
    let app = build_app();

    let genesis = app.genesis().await;
    let (block1, _result1) = app.propose(&genesis, 1).await.unwrap();
    let (block2, _result2) = app.propose(&block1, 2).await.unwrap();

    assert_eq!(genesis.height, 0);
    assert_eq!(block1.height, 1);
    assert_eq!(block2.height, 2);

    assert_eq!(block1.parent_id, genesis.compute_id());
    assert_eq!(block2.parent_id, block1.compute_id());

    // MVP behavior with empty execution: state roots remain unchanged across blocks.
    assert_eq!(genesis.state_root, block1.state_root);
    assert_eq!(block1.state_root, block2.state_root);
}

#[tokio::test]
async fn test_failed_verify_does_not_corrupt_state() {
    let app = build_app();

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
