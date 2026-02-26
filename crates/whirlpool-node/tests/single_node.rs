use std::num::NonZeroUsize;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use consensus::ConsensusEngine;
use consensus_simplex::{CommonwareConfig, CommonwareEngine, FinalizationSink};
use p2p::mock::MockNetworkProvider;
use whirlpool_node::app::EmptyBlockApp;
use whirlpool_node::block::EmptyBlock;

#[tokio::test]
async fn test_single_node_finalizes_blocks() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let app = Arc::new(EmptyBlockApp::new());
    let _height = Arc::new(AtomicU64::new(0));
    let sink = Arc::new(FinalizationSink::<EmptyBlock>::new(Arc::clone(&_height)));

    let config = CommonwareConfig {
        namespace: "single-node-test".to_string(),
        leader_timeout: Duration::from_secs(1),
        notarization_timeout: Duration::from_secs(1),
        nullify_retry: Duration::from_millis(100),
        activity_timeout: 10,
        skip_timeout: 5,
        mailbox_size: 16,
        replay_buffer: NonZeroUsize::new(16).unwrap(),
        write_buffer: NonZeroUsize::new(16).unwrap(),
        epoch: 0,
        fetch_timeout: Duration::from_secs(1),
        fetch_concurrent: 4,
    };


    // Create network provider (mock for now)
    let peer_id = p2p::mock::MockPeerId(0);
    let network = MockNetworkProvider::new(peer_id);

    let engine = CommonwareEngine::new(app, sink, config, network);
    let running = engine.start().expect("engine should start");
    assert!(running.status().is_running);

    let timeout = Duration::from_secs(30);
    let started = std::time::Instant::now();
    while started.elapsed() < timeout {
        if running.status().current_height >= 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let final_height = running.status().current_height;
    let reached_height = final_height >= 2;

    running.shutdown().await.expect("shutdown should succeed");
    assert!(
        reached_height,
        "Expected height >= 2, got {}",
        final_height
    );
}
