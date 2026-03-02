use std::sync::{Arc, RwLock};

use alloy_primitives::B256;
use app::{Application, NoopTxSource};
use app_evm::executor::EvmApplication;
use app_evm::{WhirlpoolEvmConfig, build_sahara_chain_spec};
use state::InMemoryStateDb;

fn build_app() -> EvmApplication<InMemoryStateDb> {
    let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));
    let evm_config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let tx_source = Arc::new(NoopTxSource);
    EvmApplication::new(evm_config, state_db, tx_source)
}

#[tokio::test]
async fn test_execute_empty_block() {
    let app = build_app();
    let genesis = app.genesis().await;

    let (_block, result) = app.propose(&genesis, 1).await.unwrap();

    assert_eq!(result.gas_used, 0);
    assert_eq!(result.receipt_count, 0);
}

#[tokio::test]
async fn test_state_root_computation() {
    let app = build_app();
    let genesis = app.genesis().await;

    let (block, result) = app.propose(&genesis, 1).await.unwrap();

    assert_eq!(block.state_root, result.state_root);
}

#[tokio::test]
async fn test_reconstruct_header_for_verify() {
    let app = build_app();
    let genesis = app.genesis().await;

    let (block1, _result) = app.propose(&genesis, 1).await.unwrap();

    let verify_result = app.verify(&genesis, &block1).await;
    assert!(verify_result.is_ok());
}
