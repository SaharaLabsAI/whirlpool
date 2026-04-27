use std::sync::{Arc, RwLock};

use app::{traits::Application, ApplicationAdapter, EvmBlock, NoopTxSource};
use app_evm_execution::EvmApplication;
use app_evm_execution::WhirlpoolEvmConfig;
use app_evm_state::InMemoryStateDb;
use chainspec::build_sahara_chain_spec;
use consensus::{traits::ConsensusApp, ConsensusError};

fn assert_application_impl<A: Application<Block = EvmBlock>>(_app: &A) {}

fn build_adapter() -> ApplicationAdapter<EvmApplication<InMemoryStateDb>> {
    let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));
    let evm_config = WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec()));
    let tx_source = Arc::new(NoopTxSource);
    let evm_app = EvmApplication::new(evm_config, state_db, tx_source);
    assert_application_impl(&evm_app);
    ApplicationAdapter::new(evm_app)
}

#[tokio::test]
async fn test_adapter_propose_success() {
    let adapter = build_adapter();

    let genesis = adapter.genesis().await;
    let proposed = adapter.propose(&genesis, 1).await;

    assert!(matches!(proposed, Some(EvmBlock { .. })));
}

#[tokio::test]
async fn test_adapter_propose_returns_some() {
    let adapter = build_adapter();

    let genesis = adapter.genesis().await;
    let block = adapter
        .propose(&genesis, 1)
        .await
        .expect("adapter should wrap Application::propose Ok as Some");

    assert_eq!(block.height, 1);
    assert_eq!(block.parent_id, genesis.compute_id());
}

#[tokio::test]
async fn test_adapter_verify_success() {
    let adapter = build_adapter();

    let genesis = adapter.genesis().await;
    let block1 = adapter
        .propose(&genesis, 1)
        .await
        .expect("propose should return a block");

    let verify_result = adapter.verify(&genesis, &block1).await;
    assert!(verify_result.is_ok());
}

#[tokio::test]
async fn test_adapter_verify_failure() {
    let adapter = build_adapter();

    let genesis = adapter.genesis().await;
    let block1 = adapter
        .propose(&genesis, 1)
        .await
        .expect("propose should return a block");

    let mut tampered_block = block1.clone();
    tampered_block.state_root[0] ^= 0x01;

    let verify_result = adapter.verify(&genesis, &tampered_block).await;
    assert!(matches!(
        verify_result,
        Err(ConsensusError::InvalidBlock(_))
    ));
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
async fn test_state_root_mismatch_maps_to_consensus_invalid_block() {
    let adapter = build_adapter();

    let genesis = adapter.genesis().await;
    let block1 = adapter
        .propose(&genesis, 1)
        .await
        .expect("propose should return a block");

    let mut tampered_block = block1.clone();
    tampered_block.state_root[0] ^= 0x01;

    let adapter_verify = adapter.verify(&genesis, &tampered_block).await;
    assert!(matches!(
        adapter_verify,
        Err(ConsensusError::InvalidBlock(_))
    ));
}
