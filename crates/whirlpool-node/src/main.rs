use consensus::ConsensusEngine;
use consensus_simplex::{CommonwareConfig, CommonwareEngine, FinalizationSink};
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::EnvFilter;
use whirlpool_node::app::EmptyBlockApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Init tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("whirlpool-node starting");

    // 2. Create consensus app and sink
    let app = Arc::new(EmptyBlockApp::new());
    let height = Arc::new(AtomicU64::new(0));
    let sink = Arc::new(FinalizationSink::new(Arc::clone(&height)));

    // 3. Configure commonware engine
    let config = CommonwareConfig {
        namespace: String::from_utf8_lossy(whirlpool_node::config::NAMESPACE).to_string(),
        leader_timeout: Duration::from_secs(5),
        notarization_timeout: Duration::from_secs(5),
        nullify_retry: Duration::from_millis(500),
        activity_timeout: 10,
        skip_timeout: 5,
        mailbox_size: 100,
        replay_buffer: NonZeroUsize::new(100).unwrap(),
        write_buffer: NonZeroUsize::new(100).unwrap(),
        epoch: 0,
        fetch_timeout: Duration::from_secs(5),
        fetch_concurrent: 4,
    };

    // 4. Create and start the engine
    let engine = CommonwareEngine::new(app, sink, config);
    let running = engine.start().expect("failed to start consensus engine");

    tracing::info!("consensus engine started, press Ctrl-C to stop");

    // 5. Wait for Ctrl-C
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c");

    tracing::info!("shutting down...");

    // 6. Shutdown gracefully
    running.shutdown().await.expect("failed to shutdown engine");

    tracing::info!("whirlpool-node stopped");

    Ok(())
}
